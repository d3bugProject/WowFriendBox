-- =========================================================================
-- PartySync - Addon Multiboxing WoW 3.3.5a (Warmane)
-- Gestionnaire de groupe, Choix de Roles (Tank/DPS/Heal),
-- Grille de Soins (Click-to-Heal Pretre), Auto-Quetes, Auto-Invite, Auto-Taxi
-- =========================================================================

local ADDON_NAME = "partysync"
local ADDON_PREFIX = "PS_SYNC"

-- Default database
local defaultDB = {
    members = { "", "", "", "", "" },
    roles = { "TANK", "DPS", "DPS", "HEAL", "DPS" },
    auto_invite = true,
    auto_quest = true,
    auto_taxi = true,
    auto_release = false,
    show_heal_grid = true,
    lock_heal_grid = false,
    grid_pos = { x = 20, y = 280 },
}

-- Ensure PartySyncDB is NEVER nil anywhere
if not PartySyncDB then
    PartySyncDB = defaultDB
end

-- Role metadata
local ROLE_INFO = {
    TANK = { label = "[TANK]", color = "|cff00aaff", icon = "Interface\\Icons\\Ability_Defend", tag = "[T]" },
    DPS  = { label = "[DPS]",  color = "|cffff4444", icon = "Interface\\Icons\\Ability_DualWield", tag = "[D]" },
    HEAL = { label = "[HEAL]", color = "|cff44ff44", icon = "Interface\\Icons\\Spell_Holy_FlashHeal", tag = "[H]" },
}

local ROLE_CYCLE = { "TANK", "DPS", "HEAL" }

local function GetNextRole(current)
    if current == "TANK" then return "DPS"
    elseif current == "DPS" then return "HEAL"
    else return "TANK" end
end

-- Forward declaration of frames
local f = CreateFrame("Frame", "PartySyncMainFrame", UIParent)
local healFrame = CreateFrame("Frame", "PartySyncHealFrame", UIParent)

-- =========================================================================
-- 1. MAIN CONFIGURATION WINDOW (/ps)
-- =========================================================================
f:SetSize(380, 410)
f:SetPoint("CENTER", UIParent, "CENTER", 0, 100)
f:SetBackdrop({
    bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background-Dark",
    edgeFile = "Interface\\Tooltips\\UI-Tooltip-Border",
    tile = true, tileSize = 16, edgeSize = 16,
    insets = { left = 4, right = 4, top = 4, bottom = 4 }
})
f:SetBackdropColor(0.08, 0.09, 0.12, 0.95)
f:SetBackdropBorderColor(0, 0.8, 1, 0.8)
f:SetMovable(true)
f:EnableMouse(true)
f:RegisterForDrag("LeftButton")
f:SetScript("OnDragStart", f.StartMoving)
f:SetScript("OnDragStop", f.StopMovingOrSizing)
f:Hide()

-- Title Bar
local title = f:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
title:SetPoint("TOP", f, "TOP", 0, -12)
title:SetText("|cff00ddff[PartySync]|r - Multibox Manager")

local sub = f:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
sub:SetPoint("TOP", title, "BOTTOM", 0, -4)
sub:SetText("Noms des 5 personnages et selection des Roles :")

-- Member Edit Boxes & Role Buttons
local editBoxes = {}
local roleButtons = {}
local defaultRoleLabels = { "Slot 1 (Main Tank)", "Slot 2 (DPS 1)", "Slot 3 (DPS 2)", "Slot 4 (DPS 3)", "Slot 5 (Healer)" }

for i = 1, 5 do
    local lbl = f:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    lbl:SetPoint("TOPLEFT", f, "TOPLEFT", 18, -45 - (i - 1) * 40)
    lbl:SetText("|cffaaaaaa" .. defaultRoleLabels[i] .. " :|r")

    local eb = CreateFrame("EditBox", "PSEditBox" .. i, f, "InputBoxTemplate")
    eb:SetSize(190, 20)
    eb:SetPoint("TOPLEFT", lbl, "BOTTOMLEFT", 6, -2)
    eb:SetAutoFocus(false)
    eb:SetScript("OnTextChanged", function(self)
        if PartySyncDB and PartySyncDB.members then
            PartySyncDB.members[i] = self:GetText():trim()
        end
    end)
    eb:SetScript("OnEnterPressed", function(self) self:ClearFocus() end)
    editBoxes[i] = eb

    local rb = CreateFrame("Button", "PSRoleBtn" .. i, f, "UIPanelButtonTemplate")
    rb:SetSize(75, 22)
    rb:SetPoint("LEFT", eb, "RIGHT", 10, 0)
    rb:SetScript("OnClick", function(self)
        if PartySyncDB and PartySyncDB.roles then
            local nextR = GetNextRole(PartySyncDB.roles[i] or "DPS")
            PartySyncDB.roles[i] = nextR
            local info = ROLE_INFO[nextR] or ROLE_INFO.DPS
            self:SetText(info.color .. info.label .. "|r")
            if PartySync_UpdateHealGrid then PartySync_UpdateHealGrid() end
        end
    end)
    roleButtons[i] = rb
end

-- Helper for checkboxes
local function CreateCheckbox(parent, labelText, yOffset, dbKey, onClickExtra)
    local cb = CreateFrame("CheckButton", nil, parent, "UICheckButtonTemplate")
    cb:SetPoint("TOPLEFT", parent, "TOPLEFT", 18, yOffset)
    local cbLabel = cb:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    cbLabel:SetPoint("LEFT", cb, "RIGHT", 4, 1)
    cbLabel:SetText(labelText)
    cb:SetScript("OnClick", function(self)
        PartySyncDB[dbKey] = self:GetChecked() and true or false
        if onClickExtra then onClickExtra(PartySyncDB[dbKey]) end
    end)
    return cb
end

local cbQuest = CreateCheckbox(f, "|cff00ff88Auto-Quetes|r (Accepter / Rendre)", -250, "auto_quest")
local cbInvite = CreateCheckbox(f, "|cff00ddffAuto-Invite|r (Accepter invitations du groupe)", -272, "auto_invite")
local cbTaxi = CreateCheckbox(f, "|cffffcc00Auto-Taxi|r (Vols synchronises)", -294, "auto_taxi")
local cbGrid = CreateCheckbox(f, "|cff44ff44Grille de Soins|r (Afficher Heal Grid)", -316, "show_heal_grid", function(checked)
    if checked then healFrame:Show() else healFrame:Hide() end
end)
local cbLock = CreateCheckbox(f, "|cffffaa00Verrouiller|r la position de la grille de soins", -338, "lock_heal_grid")

-- Helper for buttons
local function CreateStyledButton(parent, text, w, h, onClick)
    local btn = CreateFrame("Button", nil, parent, "UIPanelButtonTemplate")
    btn:SetSize(w, h)
    btn:SetText(text)
    btn:SetScript("OnClick", onClick)
    return btn
end

-- Invite All Button
local btnInvite = CreateStyledButton(f, "Inviter Groupe (5)", 140, 24, function()
    local myName = UnitName("player"):lower()
    local count = 0
    for _, name in ipairs(PartySyncDB.members or {}) do
        local n = name:trim()
        if n ~= "" and n:lower() ~= myName then
            InviteUnit(n)
            count = count + 1
        end
    end
    print("|cff00ddff[PartySync]|r Invitation envoyee a " .. count .. " personnage(s).")
end)
btnInvite:SetPoint("BOTTOMLEFT", f, "BOTTOMLEFT", 18, 14)

-- Disband Party Button
local btnDisband = CreateStyledButton(f, "Dissoudre", 80, 24, function()
    LeaveParty()
    print("|cff00ddff[PartySync]|r Groupe dissous.")
end)
btnDisband:SetPoint("LEFT", btnInvite, "RIGHT", 6, 0)

-- Close Button
local btnClose = CreateStyledButton(f, "X", 22, 22, function() f:Hide() end)
btnClose:SetPoint("TOPRIGHT", f, "TOPRIGHT", -8, -8)

-- =========================================================================
-- 2. HEAL GRID FRAME (Interface Barres de Vie & Click-to-Heal)
-- =========================================================================
healFrame:SetSize(200, 225)
healFrame:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 20, 280)
healFrame:SetBackdrop({
    bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background-Dark",
    edgeFile = "Interface\\Tooltips\\UI-Tooltip-Border",
    tile = true, tileSize = 16, edgeSize = 12,
    insets = { left = 3, right = 3, top = 3, bottom = 3 }
})
healFrame:SetBackdropColor(0.06, 0.07, 0.10, 0.90)
healFrame:SetBackdropBorderColor(0, 0.7, 1, 0.7)
healFrame:SetMovable(true)
healFrame:EnableMouse(true)
healFrame:RegisterForDrag("LeftButton")

healFrame:SetScript("OnDragStart", function(self)
    if not (PartySyncDB and PartySyncDB.lock_heal_grid) then
        self:StartMoving()
    end
end)

healFrame:SetScript("OnDragStop", function(self)
    self:StopMovingOrSizing()
    local x, y = self:GetLeft(), self:GetBottom()
    if PartySyncDB then
        PartySyncDB.grid_pos = { x = x, y = y }
    end
end)

-- Header Bar
local hTitle = healFrame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
hTitle:SetPoint("TOPLEFT", healFrame, "TOPLEFT", 8, -6)
hTitle:SetText("|cff00ddffPartySync Soins|r")

local hLockBtn = CreateFrame("Button", nil, healFrame, "UIPanelButtonTemplate")
hLockBtn:SetSize(20, 18)
hLockBtn:SetPoint("TOPRIGHT", healFrame, "TOPRIGHT", -26, -4)
hLockBtn:SetText("L")
hLockBtn:SetScript("OnClick", function()
    PartySyncDB.lock_heal_grid = not PartySyncDB.lock_heal_grid
    cbLock:SetChecked(PartySyncDB.lock_heal_grid)
    local state = PartySyncDB.lock_heal_grid and "|cff00ff00Verrouillee|r" or "|cffffaa00Deverrouillee|r"
    print("|cff00ddff[PartySync]|r Grille de soins " .. state)
end)

local hCloseBtn = CreateFrame("Button", nil, healFrame, "UIPanelButtonTemplate")
hCloseBtn:SetSize(18, 18)
hCloseBtn:SetPoint("TOPRIGHT", healFrame, "TOPRIGHT", -5, -4)
hCloseBtn:SetText("X")
hCloseBtn:SetScript("OnClick", function()
    healFrame:Hide()
    PartySyncDB.show_heal_grid = false
    cbGrid:SetChecked(false)
end)

-- Class Healing Spells detection
local function GetHealerSpells()
    local _, playerClass = UnitClass("player")
    local s1, s2 = "Flash Heal", "Power Word: Shield"
    if playerClass == "PRIEST" then
        s1 = GetSpellInfo(2061) or "Flash Heal"               -- Soins rapides
        s2 = GetSpellInfo(17) or "Power Word: Shield"         -- Mot de pouvoir : Bouclier
    elseif playerClass == "PALADIN" then
        s1 = GetSpellInfo(19750) or "Flash of Light"          -- Eclair lumineux
        s2 = GetSpellInfo(635) or "Holy Light"                -- Lumiere sacree
    elseif playerClass == "SHAMAN" then
        s1 = GetSpellInfo(8004) or "Lesser Healing Wave"      -- Vague de soins inferieurs
        s2 = GetSpellInfo(331) or "Healing Wave"              -- Vague de soins
    elseif playerClass == "DRUID" then
        s1 = GetSpellInfo(8936) or "Regrowth"                 -- Retablissement
        s2 = GetSpellInfo(774) or "Rejuvenation"              -- Recuperation
    end
    return s1, s2
end

-- 5 Health Bars Elements
local healBars = {}

for i = 1, 5 do
    local barContainer = CreateFrame("Frame", "PSBarContainer" .. i, healFrame)
    barContainer:SetSize(184, 34)
    barContainer:SetPoint("TOPLEFT", healFrame, "TOPLEFT", 8, -26 - (i - 1) * 38)

    -- Status Bar (Health)
    local hpBar = CreateFrame("StatusBar", "PSHealthBar" .. i, barContainer)
    hpBar:SetSize(184, 26)
    hpBar:SetPoint("TOPLEFT", barContainer, "TOPLEFT", 0, 0)
    hpBar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
    hpBar:SetMinMaxValues(0, 100)
    hpBar:SetValue(100)
    hpBar:SetStatusBarColor(0.1, 0.85, 0.2, 0.9)

    -- Background for health bar
    local hpBg = hpBar:CreateTexture(nil, "BACKGROUND")
    hpBg:SetAllPoints(hpBar)
    hpBg:SetTexture(0.15, 0.15, 0.15, 0.8)

    -- Power Bar (Mana / Energy / Rage)
    local mpBar = CreateFrame("StatusBar", "PSPowerBar" .. i, barContainer)
    mpBar:SetSize(184, 5)
    mpBar:SetPoint("TOPLEFT", hpBar, "BOTTOMLEFT", 0, -1)
    mpBar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
    mpBar:SetMinMaxValues(0, 100)
    mpBar:SetValue(100)
    mpBar:SetStatusBarColor(0, 0.4, 0.9, 0.9)

    local mpBg = mpBar:CreateTexture(nil, "BACKGROUND")
    mpBg:SetAllPoints(mpBar)
    mpBg:SetTexture(0.08, 0.08, 0.08, 0.8)

    -- Texts
    local leftText = hpBar:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    leftText:SetPoint("LEFT", hpBar, "LEFT", 4, 0)
    leftText:SetText("|cffaaaaaaSlot " .. i .. "|r")

    local rightText = hpBar:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    rightText:SetPoint("RIGHT", hpBar, "RIGHT", -4, 0)
    rightText:SetText("100%")

    -- Clickable Secure Button (for Healer client) or Action Button
    local clickBtn = CreateFrame("Button", "PSHealBtn" .. i, barContainer, "SecureActionButtonTemplate")
    clickBtn:SetAllPoints(barContainer)
    clickBtn:RegisterForClicks("AnyUp", "AnyDown")

    -- Flash feedback on click (Tank or Healer)
    clickBtn:SetScript("PreClick", function(self, button)
        -- Visual feedback
        hpBar:SetAlpha(0.5)
    end)
    clickBtn:SetScript("PostClick", function(self, button)
        hpBar:SetAlpha(1.0)
    end)

    healBars[i] = {
        container = barContainer,
        hpBar = hpBar,
        mpBar = mpBar,
        leftText = leftText,
        rightText = rightText,
        clickBtn = clickBtn,
        unit = nil,
    }
end

-- Resolve member name to unit ID ("player", "party1".."party4")
local function ResolveUnit(targetName)
    if not targetName or targetName == "" then return nil end
    local clean = targetName:lower():trim()
    local myName = UnitName("player"):lower()
    if clean == myName then
        return "player"
    end
    for p = 1, GetNumPartyMembers() do
        local pName = UnitName("party" .. p)
        if pName and pName:lower() == clean then
            return "party" .. p
        end
    end
    return nil
end

-- Configure Secure Buttons attributes (only outside combat!)
local function ConfigureHealButtons()
    if InCombatLockdown() then return end

    local s1, s2 = GetHealerSpells()

    for i = 1, 5 do
        local bar = healBars[i]
        local memberName = (PartySyncDB and PartySyncDB.members and PartySyncDB.members[i]) or ""
        local unitID = ResolveUnit(memberName)
        bar.unit = unitID

        if unitID then
            bar.clickBtn:SetAttribute("type1", "spell")
            bar.clickBtn:SetAttribute("spell1", s1)
            bar.clickBtn:SetAttribute("type2", "spell")
            bar.clickBtn:SetAttribute("spell2", s2)
            bar.clickBtn:SetAttribute("unit", unitID)
        else
            bar.clickBtn:SetAttribute("type1", nil)
            bar.clickBtn:SetAttribute("type2", nil)
            bar.clickBtn:SetAttribute("unit", nil)
        end
    end
end

-- Update Health & Mana values
function PartySync_UpdateHealGrid()
    if not PartySyncDB then return end

    for i = 1, 5 do
        local bar = healBars[i]
        local memberName = PartySyncDB.members[i] or ""
        local role = (PartySyncDB.roles and PartySyncDB.roles[i]) or "DPS"
        local rInfo = ROLE_INFO[role] or ROLE_INFO.DPS
        local unitID = ResolveUnit(memberName)
        bar.unit = unitID

        if unitID and UnitExists(unitID) then
            local curHp = UnitHealth(unitID)
            local maxHp = UnitHealthMax(unitID)
            local pct = (maxHp > 0) and math.floor((curHp / maxHp) * 100) or 0

            bar.hpBar:SetMinMaxValues(0, maxHp > 0 and maxHp or 1)
            bar.hpBar:SetValue(curHp)

            -- Color transition based on health %
            if pct > 60 then
                bar.hpBar:SetStatusBarColor(0.1, 0.85, 0.2, 0.9) -- Green
            elseif pct > 30 then
                bar.hpBar:SetStatusBarColor(0.9, 0.8, 0.1, 0.9)  -- Yellow
            else
                bar.hpBar:SetStatusBarColor(0.95, 0.15, 0.15, 0.9) -- Red
            end

            -- Dead / Ghost status
            if UnitIsDead(unitID) or UnitIsGhost(unitID) then
                bar.leftText:SetText(rInfo.color .. rInfo.tag .. " " .. memberName .. "|r")
                bar.rightText:SetText("|cffff0000[MORT]|r")
                bar.hpBar:SetStatusBarColor(0.3, 0.3, 0.3, 0.8)
            else
                bar.leftText:SetText(rInfo.color .. rInfo.tag .. " " .. memberName .. "|r")
                bar.rightText:SetText(pct .. "%")
            end

            -- Power Bar
            local curMp = UnitMana(unitID)
            local maxMp = UnitManaMax(unitID)
            bar.mpBar:SetMinMaxValues(0, maxMp > 0 and maxMp or 1)
            bar.mpBar:SetValue(curMp)

            local pType = UnitPowerType(unitID)
            if pType == 0 then
                bar.mpBar:SetStatusBarColor(0, 0.45, 0.9, 0.9)   -- Mana (Blue)
            elseif pType == 1 then
                bar.mpBar:SetStatusBarColor(0.9, 0.1, 0.1, 0.9)   -- Rage (Red)
            elseif pType == 3 then
                bar.mpBar:SetStatusBarColor(0.95, 0.85, 0.1, 0.9) -- Energy (Yellow)
            else
                bar.mpBar:SetStatusBarColor(0, 0.8, 0.8, 0.9)
            end
        else
            -- Unit not in group / offline
            bar.hpBar:SetMinMaxValues(0, 100)
            bar.hpBar:SetValue(0)
            bar.mpBar:SetValue(0)
            local displayName = (memberName ~= "") and memberName or ("Slot " .. i)
            bar.leftText:SetText(rInfo.color .. rInfo.tag .. " " .. displayName .. "|r")
            bar.rightText:SetText("|cff666666Off|r")
            bar.hpBar:SetStatusBarColor(0.2, 0.2, 0.2, 0.6)
        end
    end
end

-- Refresh UI from DB
local function RefreshUI()
    if not PartySyncDB then PartySyncDB = defaultDB end
    for k, v in pairs(defaultDB) do
        if PartySyncDB[k] == nil then PartySyncDB[k] = v end
    end
    for i = 1, 5 do
        editBoxes[i]:SetText(PartySyncDB.members[i] or "")
        local role = PartySyncDB.roles[i] or "DPS"
        local info = ROLE_INFO[role] or ROLE_INFO.DPS
        roleButtons[i]:SetText(info.color .. info.label .. "|r")
    end
    cbQuest:SetChecked(PartySyncDB.auto_quest)
    cbInvite:SetChecked(PartySyncDB.auto_invite)
    cbTaxi:SetChecked(PartySyncDB.auto_taxi)
    cbGrid:SetChecked(PartySyncDB.show_heal_grid)
    cbLock:SetChecked(PartySyncDB.lock_heal_grid)

    if PartySyncDB.grid_pos and PartySyncDB.grid_pos.x and PartySyncDB.grid_pos.y then
        healFrame:ClearAllPoints()
        healFrame:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", PartySyncDB.grid_pos.x, PartySyncDB.grid_pos.y)
    end

    if PartySyncDB.show_heal_grid then
        healFrame:Show()
    else
        healFrame:Hide()
    end

    ConfigureHealButtons()
    PartySync_UpdateHealGrid()
end

-- =========================================================================
-- 3. EVENT ENGINE
-- =========================================================================
local eventFrame = CreateFrame("Frame")
eventFrame:RegisterEvent("ADDON_LOADED")
eventFrame:RegisterEvent("PLAYER_LOGIN")
eventFrame:RegisterEvent("PARTY_MEMBERS_CHANGED")
eventFrame:RegisterEvent("RAID_ROSTER_UPDATE")
eventFrame:RegisterEvent("PLAYER_REGEN_ENABLED")
eventFrame:RegisterEvent("UNIT_HEALTH")
eventFrame:RegisterEvent("UNIT_MAXHEALTH")
eventFrame:RegisterEvent("UNIT_MANA")
eventFrame:RegisterEvent("UNIT_MAXMANA")
eventFrame:RegisterEvent("UNIT_RAGE")
eventFrame:RegisterEvent("UNIT_ENERGY")
eventFrame:RegisterEvent("PARTY_INVITE_REQUEST")
eventFrame:RegisterEvent("QUEST_DETAIL")
eventFrame:RegisterEvent("QUEST_ACCEPTED")
eventFrame:RegisterEvent("QUEST_GREETING")
eventFrame:RegisterEvent("QUEST_PROGRESS")
eventFrame:RegisterEvent("QUEST_COMPLETE")
eventFrame:RegisterEvent("QUEST_ACCEPT_CONFIRM")
eventFrame:RegisterEvent("GOSSIP_SHOW")
eventFrame:RegisterEvent("TAXIMAP_OPENED")
eventFrame:RegisterEvent("CHAT_MSG_ADDON")

-- Fast ticker for smooth HP updates & auto-accepting shared quests
local elapsedTimer = 0
eventFrame:SetScript("OnUpdate", function(self, elapsed)
    elapsedTimer = elapsedTimer + elapsed
    if elapsedTimer >= 0.25 then
        elapsedTimer = 0
        if PartySyncDB and PartySyncDB.show_heal_grid and healFrame:IsShown() then
            PartySync_UpdateHealGrid()
        end
        -- Auto-acceptation des quetes partagees par le groupe
        if PartySyncDB and PartySyncDB.auto_quest then
            if StaticPopup_Visible("QUEST_ACCEPT") then
                AcceptQuest()
                StaticPopup_Hide("QUEST_ACCEPT")
                print("|cff00ddff[PartySync]|r Quete partagee par le groupe acceptee automatiquement !")
            end
            if StaticPopup_Visible("QUEST_ACCEPT_CONFIRM") then
                ConfirmAcceptQuest()
                StaticPopup_Hide("QUEST_ACCEPT_CONFIRM")
                print("|cff00ddff[PartySync]|r Quete d'escorte groupee acceptee automatiquement !")
            end
        end
    end
end)

local function IsGroupMember(inviter)
    if not inviter then return false end
    local clean = inviter:lower():trim()
    for _, m in ipairs(PartySyncDB.members or {}) do
        if m:lower():trim() == clean then
            return true
        end
    end
    return false
end

eventFrame:SetScript("OnEvent", function(self, event, ...)
    local arg1, arg2, arg3 = ...

    -- 1. Load DB
    if (event == "ADDON_LOADED" and arg1 and arg1:lower() == "partysync") or event == "PLAYER_LOGIN" then
        if not PartySyncDB then PartySyncDB = defaultDB end
        RefreshUI()
        if RegisterAddonMessagePrefix then
            RegisterAddonMessagePrefix(ADDON_PREFIX)
        end
        -- Configuration automatique des touches F, G et du Clic pour se deplacer
        local myName = UnitName("player") and UnitName("player"):lower() or ""
        if myName == "unoz" then
            SetCVar("autointeract", "0") -- Pas de deplacement au clic sur le Tank (Unoz)
        else
            SetCVar("autointeract", "1") -- Actif sur les 4 esclaves
        end
        SetBinding("F", "ASSISTTARGET")
        SetBinding("G", "INTERACTTARGET")
        SaveBindings(GetCurrentBindingSet())
        print("|cff00ddff[PartySync]|r Touches [F] et [G] configurees ! (Clic deplacement desactive sur Unoz)")

    -- 2. Party change or combat drop: update heal buttons
    elseif event == "PARTY_MEMBERS_CHANGED" or event == "RAID_ROSTER_UPDATE" or event == "PLAYER_REGEN_ENABLED" then
        ConfigureHealButtons()
        PartySync_UpdateHealGrid()

    -- 3. Unit Health / Mana events
    elseif event == "UNIT_HEALTH" or event == "UNIT_MAXHEALTH" or event == "UNIT_MANA" or event == "UNIT_MAXMANA" or event == "UNIT_RAGE" or event == "UNIT_ENERGY" then
        PartySync_UpdateHealGrid()

    -- 4. Auto-Accept Party Invites
    elseif event == "PARTY_INVITE_REQUEST" then
        local inviter = arg1
        if PartySyncDB.auto_invite and IsGroupMember(inviter) then
            AcceptGroup()
            StaticPopup_Hide("PARTY_INVITE")
            print("|cff00ddff[PartySync]|r Invitation de |cff00ff88" .. inviter .. "|r acceptee automatiquement !")
        end

    -- 5. Auto-Accept Quests
    elseif event == "QUEST_DETAIL" then
        if PartySyncDB.auto_quest then
            AcceptQuest()
            print("|cff00ddff[PartySync]|r Quete acceptee : |cffffff00" .. (GetTitleText() or "") .. "|r")
        end

    -- 5b. Auto-Share Quests to Party
    elseif event == "QUEST_ACCEPTED" then
        if PartySyncDB.auto_quest then
            if GetNumPartyMembers() == 0 then
                print("|cffff4444[PartySync]|r Vous n'etes pas dans un groupe ! Tapez /ps -> 'Inviter Groupe (5)'.")
            else
                local questIndex = arg1 or GetNumQuestLogEntries()
                if questIndex and questIndex > 0 then
                    SelectQuestLogEntry(questIndex)
                    if GetQuestLogPushable() then
                        QuestLogPushQuest()
                        print("|cff00ddff[PartySync]|r Quete partagee au groupe !")
                    else
                        print("|cffffaa00[PartySync]|r Cette quete est solo/intro (non-partageable par WoW). Cible le PNJ avec Unoz, puis appuie sur F puis G !")
                    end
                end
            end
        end

    -- 5c. Quest Greeting (Multiple quests at NPC)
    elseif event == "QUEST_GREETING" then
        if PartySyncDB.auto_quest and not IsShiftKeyDown() then
            local numActive = GetNumActiveQuests()
            local numAvail = GetNumAvailableQuests()
            -- Uniquement s'il n'y a qu'une seule quete disponible et aucune active : on l'ouvre automatiquement
            if numAvail == 1 and numActive == 0 then
                SelectAvailableQuest(1)
            -- S'il y a plusieurs quetes ou des quetes en cours, on laisse la liste ouverte pour pouvoir choisir librement !
            end
        end

    -- 6. Auto-Progress Quests
    elseif event == "QUEST_PROGRESS" then
        if PartySyncDB.auto_quest and IsQuestCompletable() then
            CompleteQuest()
        end

    -- 7. Auto-Complete Quests with best vendor price
    elseif event == "QUEST_COMPLETE" then
        if PartySyncDB.auto_quest then
            local numChoices = GetNumQuestChoices()
            if numChoices > 1 then
                local bestIdx = 1
                local bestPrice = 0
                for i = 1, numChoices do
                    local link = GetQuestItemLink("choice", i)
                    if link then
                        local _, _, _, _, _, _, _, _, _, _, vendorPrice = GetItemInfo(link)
                        if vendorPrice and vendorPrice > bestPrice then
                            bestPrice = vendorPrice
                            bestIdx = i
                        end
                    end
                end
                GetQuestReward(bestIdx)
            else
                GetQuestReward(1)
            end
            print("|cff00ddff[PartySync]|r Quete validee et rendue !")
        end

    -- 8. Auto-Confirm Escort / Shared Quest
    elseif event == "QUEST_ACCEPT_CONFIRM" then
        if PartySyncDB.auto_quest then
            ConfirmAcceptQuest()
            StaticPopup_Hide("QUEST_ACCEPT")
        end

    -- 9. Gossip
    elseif event == "GOSSIP_SHOW" then
        if PartySyncDB.auto_quest and not IsShiftKeyDown() then
            local numActive = GetNumGossipActiveQuests()
            local numAvail = GetNumGossipAvailableQuests()
            -- Uniquement s'il n'y a qu'une seule quete disponible et aucune active : on l'ouvre automatiquement
            if numAvail == 1 and numActive == 0 then
                SelectGossipAvailableQuest(1)
            -- S'il y a plusieurs quetes, la liste reste visible et cliquable sans blocage
            end
        end

    -- 10. Addon Messages (Taxi Sync)
    elseif event == "CHAT_MSG_ADDON" then
        local prefix, msg, channel, sender = ...
        if prefix == ADDON_PREFIX and PartySyncDB.auto_taxi then
            local cmd, param = strsplit(":", msg)
            if cmd == "TAXI" and param then
                local nodeIndex = tonumber(param)
                if nodeIndex and TaxiNodeIsOpen(nodeIndex) then
                    TakeTaxiNode(nodeIndex)
                    print("|cff00ddff[PartySync]|r Vol synchronise pris vers le noeud #" .. nodeIndex)
                end
            end
        end
    end
end)

-- Hook flight master
hooksecurefunc("TakeTaxiNode", function(nodeIndex)
    if PartySyncDB and PartySyncDB.auto_taxi and GetNumPartyMembers() > 0 then
        SendAddonMessage(ADDON_PREFIX, "TAXI:" .. nodeIndex, "PARTY")
    end
end)

-- Hook StaticPopup_Show for 0ms instantaneous auto-accept of shared quests
hooksecurefunc("StaticPopup_Show", function(which)
    if not (PartySyncDB and PartySyncDB.auto_quest) then return end
    if which == "QUEST_ACCEPT" then
        AcceptQuest()
        StaticPopup_Hide("QUEST_ACCEPT")
        print("|cff00ddff[PartySync]|r Quete partagee par le groupe acceptee automatiquement !")
    elseif which == "QUEST_ACCEPT_CONFIRM" then
        ConfirmAcceptQuest()
        StaticPopup_Hide("QUEST_ACCEPT_CONFIRM")
        print("|cff00ddff[PartySync]|r Quete d'escorte groupee acceptee automatiquement !")
    end
end)

-- =========================================================================
-- 4. SLASH COMMANDS (/ps, /partysync)
-- =========================================================================
SLASH_PARTYSYNC1 = "/ps"
SLASH_PARTYSYNC2 = "/partysync"
SlashCmdList["PARTYSYNC"] = function(msg)
    local arg = msg:lower():trim()
    if arg == "inv" or arg == "invite" then
        btnInvite:GetScript("OnClick")(btnInvite)
    elseif arg == "dis" or arg == "disband" or arg == "leave" then
        btnDisband:GetScript("OnClick")(btnDisband)
    elseif arg == "grid" or arg == "heal" then
        PartySyncDB.show_heal_grid = not PartySyncDB.show_heal_grid
        cbGrid:SetChecked(PartySyncDB.show_heal_grid)
        if PartySyncDB.show_heal_grid then healFrame:Show() else healFrame:Hide() end
    elseif arg == "lock" then
        PartySyncDB.lock_heal_grid = not PartySyncDB.lock_heal_grid
        cbLock:SetChecked(PartySyncDB.lock_heal_grid)
        local state = PartySyncDB.lock_heal_grid and "|cff00ff00Verrouillee|r" or "|cffffaa00Deverrouillee|r"
        print("|cff00ddff[PartySync]|r Grille de soins " .. state)
    else
        RefreshUI()
        if f:IsShown() then
            f:Hide()
        else
            f:Show()
        end
    end
end
