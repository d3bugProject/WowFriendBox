-- =========================================================================
-- EazyBox - map.lua
-- Integration et Controle direct du moteur de Carte haute precision Carbonite
-- Gère l'affichage, le zoom/pan fluide, le suivi du joueur, l'activation des
-- gisements et herbes (CarboniteNodes), et l'alerte radar clignotante a proximite.
-- =========================================================================

local alertFrame = nil
local alertText = nil
local lastAlertTime = 0
local carbDockBar = nil
local lastNearNode = nil

-- Bascule de la carte Carbonite
function EazyBox_ToggleMap(forceState)
    if not (Nx and Nx.Win and Nx.Win.CoS) then
        -- Si Carbonite n'est pas encore pret, on passe par la commande slash
        if SlashCmdList["Carbonite"] then
            SlashCmdList["Carbonite"]("winshow map")
            return
        end
        print("|cffff4444[EazyBox]|r L'addon Carbonite n'est pas active dans vos AddOns WoW.")
        return
    end

    if forceState ~= nil then
        Nx.Win:CoS("map " .. (forceState and "1" or "0"))
        if EazyBoxDB then EazyBoxDB.show_map = forceState end
    else
        Nx.Win:CoS("map")
        local isShown = EazyBox_IsCarboniteMapShown()
        if EazyBoxDB then EazyBoxDB.show_map = isShown end
    end

    -- Active automatiquement l'affichage des filons et des herbes sur la carte
    EazyBox_EnableCarboniteGathering(true, true)
end

-- Verifie si la carte Carbonite est actuellement affichee
function EazyBox_IsCarboniteMapShown()
    local mapWin = _G["NxMap1"]
    if mapWin and mapWin:IsShown() then
        return true
    end
    if Nx and Nx.Win and Nx.Win.Win and Nx.Win.Win["NxMap1"] then
        local w = Nx.Win.Win["NxMap1"]
        return w.Frm and w.Frm:IsShown()
    end
    return false
end

-- Active l'affichage des minerais et des herbes dans le moteur Carbonite
function EazyBox_EnableCarboniteGathering(showMines, showHerbs)
    if not (Nx and Nx.ChO) then return end

    if showMines ~= nil then
        Nx.ChO["MapShowGatherM"] = showMines
    end
    if showHerbs ~= nil then
        Nx.ChO["MapShowGatherH"] = showHerbs
    end

    -- Force le rafraichissement des icones de collecte sur la carte Carbonite
    if Nx.Map then
        local map = Nx.Map:GeM(1)
        if map and map.Gui and map.Gui.UGF then
            map.Gui:UGF()
        end
    end
end

-- Importation automatique ou manuelle des gisements et herbes CarboniteNodes
function EazyBox_AutoImportNodes(force)
    if not (Nx and Nx.GIC) then
        print("|cffff4444[EazyBox]|r Impossible d'importer : verifiez que Carbonite et CarboniteNodes sont actifs dans vos AddOns.")
        return
    end

    local needImport = force
    if not needImport then
        local hasMines = NxData and NxData.NXGather and NxData.NXGather["NXMine"] and next(NxData.NXGather["NXMine"])
        local hasHerbs = NxData and NxData.NXGather and NxData.NXGather["NXHerb"] and next(NxData.NXGather["NXHerb"])
        if not hasMines or not hasHerbs then
            needImport = true
        end
    end

    if needImport then
        print("|cff00ddff[EazyBox]|r Importation des gisements et herbes certifiés (CarboniteNodes)...")
        Nx:GIC("NXMine")
        Nx:GIC("NXHerb")
        print("|cff00ff00[EazyBox]|r Importation terminee avec succes ! Tous les points officiels WotLK sont affiches.")
    end

    EazyBox_EnableCarboniteGathering(true, true)
end

-- Initialisation du module Radar / Carte Carbonite
function EazyBox_InitMapModule()
    -- 1. Cadre d'alerte radar clignotante a proximite d'une ressource
    if not alertFrame then
        alertFrame = CreateFrame("Frame", "EazyBoxRadarAlertFrame", UIParent)
        alertFrame:SetSize(420, 40)
        alertFrame:SetPoint("TOP", UIParent, "TOP", 0, -180)
        alertFrame:SetFrameStrata("HIGH")
        alertFrame:Hide()

        local bg = alertFrame:CreateTexture(nil, "BACKGROUND")
        bg:SetAllPoints(alertFrame)
        bg:SetTexture("Interface\\DialogFrame\\UI-DialogBox-Background-Dark")
        bg:SetAlpha(0.85)

        local border = alertFrame:CreateTexture(nil, "BORDER")
        border:SetAllPoints(alertFrame)
        border:SetTexture("Interface\\Tooltips\\UI-Tooltip-Border")

        alertText = alertFrame:CreateFontString(nil, "OVERLAY", "GameFontHighlightLarge")
        alertText:SetPoint("CENTER", alertFrame, "CENTER", 0, 0)
        alertText:SetText("|cffffff00*** RESSOURCE PROCHE ! ***|r")
    end

    -- 2. Barre de controle EazyBox ancree sur la fenetre de carte Carbonite
    EazyBox_AttachDockBarToCarbonite()

    -- 3. Surveillance de proximite (Radar Ping toutes les 0.4s)
    local checkTimer = 0
    local pulseTimer = 0
    local proximityTicker = CreateFrame("Frame", nil, UIParent)
    proximityTicker:SetScript("OnUpdate", function(self, elapsed)
        checkTimer = checkTimer + elapsed
        pulseTimer = pulseTimer + elapsed

        -- Animation clignotante si alerte active
        if alertFrame:IsShown() then
            local alpha = 0.4 + 0.6 * math.abs(math.sin(pulseTimer * 5.0))
            alertFrame:SetAlpha(alpha)
            if GetTime() - lastAlertTime > 4.0 then
                alertFrame:Hide()
            end
        end

        -- Verification des ressources autour du joueur toutes les 0.4s
        if checkTimer >= 0.4 then
            checkTimer = 0
            EazyBox_CheckNearbyNodes()
        end
    end)

    -- Configuration initiale Carbonite si present
    if Nx and Nx.ChO then
        EazyBox_EnableCarboniteGathering(true, true)
    end
end

-- Ancrage d'une barre de boutons EazyBox sur la carte Carbonite
function EazyBox_AttachDockBarToCarbonite()
    local carbMap = _G["NxMap1"]
    if not carbMap or carbDockBar then return end

    carbDockBar = CreateFrame("Frame", "EazyBoxCarbDockBar", carbMap)
    carbDockBar:SetHeight(22)
    carbDockBar:SetPoint("TOPLEFT", carbMap, "TOPLEFT", 6, -2)
    carbDockBar:SetPoint("TOPRIGHT", carbMap, "TOPRIGHT", -24, -2)
    carbDockBar:SetFrameLevel(carbMap:GetFrameLevel() + 10)

    -- Bouton [EazyBox Menu]
    local btnMenu = CreateFrame("Button", nil, carbDockBar, "UIPanelButtonTemplate")
    btnMenu:SetSize(75, 18)
    btnMenu:SetPoint("LEFT", carbDockBar, "LEFT", 0, 0)
    btnMenu:SetText("|cff00ddff[EazyBox]|r")
    btnMenu:SetScript("OnClick", function()
        if EazyBoxMainFrame then
            if EazyBoxMainFrame:IsShown() then
                EazyBoxMainFrame:Hide()
            else
                EazyBoxMainFrame:Show()
                if EazyBox_UpdateGroupUI then EazyBox_UpdateGroupUI() end
            end
        end
    end)

    -- Bouton [Min] Toggle Minage
    local btnMin = CreateFrame("Button", nil, carbDockBar, "UIPanelButtonTemplate")
    btnMin:SetSize(42, 18)
    btnMin:SetPoint("LEFT", btnMenu, "RIGHT", 4, 0)
    btnMin:SetText("|cff00ff00Min|r")
    btnMin:SetScript("OnClick", function(self)
        if Nx and Nx.ChO then
            local newState = not Nx.ChO["MapShowGatherM"]
            Nx.ChO["MapShowGatherM"] = newState
            self:SetText(newState and "|cff00ff00Min|r" or "|cffff0000Min|r")
            if Nx.Map and Nx.Map:GeM(1) and Nx.Map:GeM(1).Gui then
                Nx.Map:GeM(1).Gui:UGF()
            end
        end
    end)

    -- Bouton [Hrb] Toggle Herbes
    local btnHrb = CreateFrame("Button", nil, carbDockBar, "UIPanelButtonTemplate")
    btnHrb:SetSize(42, 18)
    btnHrb:SetPoint("LEFT", btnMin, "RIGHT", 4, 0)
    btnHrb:SetText("|cff00ff00Hrb|r")
    btnHrb:SetScript("OnClick", function(self)
        if Nx and Nx.ChO then
            local newState = not Nx.ChO["MapShowGatherH"]
            Nx.ChO["MapShowGatherH"] = newState
            self:SetText(newState and "|cff00ff00Hrb|r" or "|cffff0000Hrb|r")
            if Nx.Map and Nx.Map:GeM(1) and Nx.Map:GeM(1).Gui then
                Nx.Map:GeM(1).Gui:UGF()
            end
        end
    end)

    -- Bouton [Import Nodes]
    local btnImport = CreateFrame("Button", nil, carbDockBar, "UIPanelButtonTemplate")
    btnImport:SetSize(60, 18)
    btnImport:SetPoint("LEFT", btnHrb, "RIGHT", 4, 0)
    btnImport:SetText("|cff00ffffImport|r")
    btnImport:SetScript("OnClick", function()
        EazyBox_AutoImportNodes(true)
    end)
    btnImport:SetScript("OnEnter", function(self)
        GameTooltip:SetOwner(self, "ANCHOR_TOP")
        GameTooltip:SetText("Importer tous les gisements & herbes officiels WotLK")
        GameTooltip:Show()
    end)
    btnImport:SetScript("OnLeave", function() GameTooltip:Hide() end)

    -- Bouton [Suivi F1]
    local btnFollow = CreateFrame("Button", nil, carbDockBar, "UIPanelButtonTemplate")
    btnFollow:SetSize(72, 18)
    btnFollow:SetPoint("LEFT", btnImport, "RIGHT", 4, 0)
    btnFollow:SetText("|cffffcc00Follow F1|r")
    btnFollow:SetScript("OnClick", function()
        local leaderIdx = EazyBoxDB and EazyBoxDB.leader_index or 1
        local leaderName = (EazyBoxDB and EazyBoxDB.members[leaderIdx] or ""):trim()
        if leaderName ~= "" then
            FollowUnit(leaderName)
            print("|cff00ddff[EazyBox]|r Suivi immediat de : |cff00ff00" .. leaderName .. "|r")
        end
    end)
end

-- Verification de la proximite des ressources et alerte radar
function EazyBox_CheckNearbyNodes()
    if not (EazyBoxDB and EazyBoxDB.radar_pulse_enabled) then return end

    local px, py = GetPlayerMapPosition("player")
    if not px or not py or (px == 0 and py == 0) then return end

    local zone = GetRealZoneText() or GetZoneText() or ""
    local nodes = EazyBox_GetZoneNodes and EazyBox_GetZoneNodes(zone) or {}
    if #nodes == 0 then return end

    local nearestNode = nil
    local nearestDist = 999999

    for i = 1, #nodes do
        local n = nodes[i]
        local dx = n.x - px
        local dy = n.y - py
        local dist = math.sqrt(dx * dx + dy * dy)
        if dist < nearestDist then
            nearestDist = dist
            nearestNode = n
        end
    end

    -- Seuil de detection a proximite (~75 metres, dist <= 0.012 en coordonnees carte)
    if nearestNode and nearestDist <= 0.012 then
        local distYards = math.floor(nearestDist * 10000)
        local typeColor = nearestNode.type == "MINE" and "|cffffcc00" or "|cff20ff20"

        alertText:SetText(string.format("%s*** %s A PROXIMITE ! (~%d m) ***|r", typeColor, nearestNode.name:upper(), distYards))
        alertFrame:Show()
        lastAlertTime = GetTime()

        if lastNearNode ~= nearestNode.name then
            lastNearNode = nearestNode.name
            PlaySoundFile("Sound\\Interface\\RaidWarning.wav")
        end
    else
        lastNearNode = nil
    end
end
