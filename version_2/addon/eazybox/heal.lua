-- =========================================================================
-- EazyBox - heal.lua
-- Interface de Soins (Heal Grid) avec barres de sante des 5 personnages,
-- Click-to-Heal securise et BOUTON « OPTIONS » integre ouvrant la fenetre principale.
-- =========================================================================

local healFrame = CreateFrame("Frame", "EazyBoxHealFrame", UIParent)
local memberBars = {}

-- Definition des sorts de soin par classe
local HEAL_SPELLS = {
    PRIEST = { left = "Soins rapides", right = "Mot de pouvoir : Bouclier" },
    SHAMAN = { left = "Vague de soins inferieurs", right = "Remous" },
    PALADIN = { left = "Eclair lumineux", right = "Guide de lumiere" },
    DRUID = { left = "Retablissement", right = "Recuperation" },
}

function EazyBox_InitHealModule()
    healFrame:SetSize(210, 245)
    local pos = (EazyBoxDB and EazyBoxDB.heal_grid_pos) or { x = 20, y = 280 }
    healFrame:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", pos.x, pos.y)

    healFrame:SetBackdrop({
        bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background-Dark",
        edgeFile = "Interface\\Tooltips\\UI-Tooltip-Border",
        tile = true, tileSize = 16, edgeSize = 12,
        insets = { left = 3, right = 3, top = 3, bottom = 3 }
    })
    healFrame:SetBackdropColor(0.05, 0.07, 0.10, 0.92)
    healFrame:SetBackdropBorderColor(0, 0.7, 1, 0.8)
    healFrame:SetMovable(true)
    healFrame:EnableMouse(true)
    healFrame:RegisterForDrag("LeftButton")

    healFrame:SetScript("OnDragStart", function(self)
        if not (EazyBoxDB and EazyBoxDB.lock_heal_grid) then
            self:StartMoving()
        end
    end)

    healFrame:SetScript("OnDragStop", function(self)
        self:StopMovingOrSizing()
        local x, y = self:GetLeft(), self:GetBottom()
        if EazyBoxDB then
            EazyBoxDB.heal_grid_pos = { x = x, y = y }
        end
    end)

    -- 1. Titre de l'interface de soin
    local title = healFrame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    title:SetPoint("TOPLEFT", healFrame, "TOPLEFT", 10, -8)
    title:SetText("|cff00ddffEazyBox|r |cff44ff44Heal|r")

    -- 2. BOUTON « OPTIONS » (Demande explicite dans mode.txt : ouvre la fenetre de config au clic)
    local btnOptions = CreateFrame("Button", "EazyBoxHealOptionsBtn", healFrame, "UIPanelButtonTemplate")
    btnOptions:SetSize(62, 18)
    btnOptions:SetPoint("TOPRIGHT", healFrame, "TOPRIGHT", -8, -6)
    btnOptions:SetText("Options")
    btnOptions:SetScript("OnClick", function()
        if EazyBoxMainFrame then
            if EazyBoxMainFrame:IsShown() then
                EazyBoxMainFrame:Hide()
            else
                EazyBoxMainFrame:Show()
                if EazyBox_RefreshMainUI then EazyBox_RefreshMainUI() end
            end
        end
    end)

    -- 3. Barres de vie des 5 personnages
    local _, playerClass = UnitClass("player")
    local classSpells = HEAL_SPELLS[playerClass] or { left = "Flash Heal", right = "Power Word: Shield" }

    for i = 1, 5 do
        local bar = CreateFrame("Button", "EazyBoxHealBar" .. i, healFrame, "SecureActionButtonTemplate")
        bar:SetSize(194, 38)
        bar:SetPoint("TOPLEFT", healFrame, "TOPLEFT", 8, -30 - (i - 1) * 42)

        -- Fond de la barre
        local bg = bar:CreateTexture(nil, "BACKGROUND")
        bg:SetAllPoints(bar)
        bg:SetTexture(0.12, 0.12, 0.15, 0.9)

        -- Barre de progression de vie
        local status = CreateFrame("StatusBar", nil, bar)
        status:SetAllPoints(bar)
        status:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
        status:SetStatusBarColor(0.2, 0.8, 0.2, 1.0)
        status:SetMinMaxValues(0, 100)
        status:SetValue(100)

        -- Nom du joueur et role
        local nameText = status:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
        nameText:SetPoint("TOPLEFT", status, "TOPLEFT", 6, -4)
        nameText:SetText("Slot #" .. i)

        -- Texte des PV
        local hpText = status:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
        hpText:SetPoint("BOTTOMRIGHT", status, "BOTTOMRIGHT", -6, 4)
        hpText:SetText("100%")

        -- Role tag
        local roleText = status:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
        roleText:SetPoint("TOPRIGHT", status, "TOPRIGHT", -6, -4)
        roleText:SetText("")

        -- Configuration du Click-to-Heal
        bar:RegisterForClicks("AnyUp")
        bar:SetAttribute("type1", "macro") -- Clic gauche
        bar:SetAttribute("type2", "macro") -- Clic droit

        memberBars[i] = {
            btn = bar,
            status = status,
            nameText = nameText,
            hpText = hpText,
            roleText = roleText,
        }
    end

    if EazyBoxDB and not EazyBoxDB.show_heal_grid then
        healFrame:Hide()
    else
        healFrame:Show()
    end

    EazyBox_UpdateHealGrid()
end

-- Met a jour les barres de vie en temps reel
function EazyBox_UpdateHealGrid()
    if not healFrame or not healFrame:IsShown() then return end

    local _, playerClass = UnitClass("player")
    local classSpells = HEAL_SPELLS[playerClass] or { left = "Flash Heal", right = "Power Word: Shield" }

    for i = 1, 5 do
        local item = memberBars[i]
        if item then
            local targetName = (EazyBoxDB and EazyBoxDB.members[i] or ""):trim()
            local role = (EazyBoxDB and EazyBoxDB.roles[i] or "DPS")
            local st = EazyBox_MemberStatus[i]

            if targetName ~= "" then
                item.nameText:SetText(targetName)
                item.roleText:SetText("|cffaaaaaa[" .. role .. "]|r")

                -- Met a jour la macro de clic sécurisée
                item.btn:SetAttribute("macrotext1", string.format("/cast [@%s] %s", targetName, classSpells.left))
                item.btn:SetAttribute("macrotext2", string.format("/cast [@%s] %s", targetName, classSpells.right))

                local hp = 100
                local maxHp = 100

                -- Verifie d'abord via l'unite WoW reelle (plus precis si a portee)
                local unit = EazyBox_GetUnitForName(targetName)
                if unit and UnitExists(unit) then
                    hp = UnitHealth(unit)
                    maxHp = UnitHealthMax(unit)
                elseif st and st.online then
                    hp = st.hp or 100
                    maxHp = st.maxHp or 100
                end

                if maxHp <= 0 then maxHp = 100 end
                local pct = math.floor((hp / maxHp) * 100)

                item.status:SetMinMaxValues(0, maxHp)
                item.status:SetValue(hp)
                item.hpText:SetText(pct .. "%")

                -- Couleur dynamique : Vert -> Jaune -> Rouge
                if pct > 65 then
                    item.status:SetStatusBarColor(0.2, 0.8, 0.2, 1.0)
                elseif pct > 35 then
                    item.status:SetStatusBarColor(0.9, 0.7, 0.1, 1.0)
                else
                    item.status:SetStatusBarColor(0.9, 0.2, 0.2, 1.0)
                end
            else
                item.nameText:SetText("Slot #" .. i .. " (Vide)")
                item.hpText:SetText("-")
                item.roleText:SetText("")
                item.status:SetValue(0)
            end
        end
    end
end
