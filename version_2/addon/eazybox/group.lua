-- =========================================================================
-- EazyBox - group.lua
-- Gestion du groupe, formulaire des membres, voyants lumineux de synchronisation,
-- auto-acceptation des invitations, et Auto-Follow F1 (SecureActionButton)
-- =========================================================================

local mainFrame = CreateFrame("Frame", "EazyBoxMainFrame", UIParent)
local editBoxes = {}
local statusDots = {}
local roleButtons = {}

local ROLE_INFO = {
    TANK = { label = "[TANK]", color = "|cff00aaff" },
    DPS  = { label = "[DPS]",  color = "|cffff4444" },
    HEAL = { label = "[HEAL]", color = "|cff44ff44" },
}

local ROLE_CYCLE = { "TANK", "DPS", "HEAL" }
local function GetNextRole(current)
    if current == "TANK" then return "DPS"
    elseif current == "DPS" then return "HEAL"
    else return "TANK" end
end

-- =========================================================================
-- 1. SECURE ACTION BUTTON POUR AUTO-FOLLOW F1 (Invisible & 100% Automatique)
-- =========================================================================
local followBtn = CreateFrame("Button", "EazyBoxFollowButton", UIParent, "SecureActionButtonTemplate")
followBtn:SetAttribute("type", "macro")

function EazyBox_UpdateFollowBinding()
    if not EazyBoxDB or not EazyBoxDB.auto_follow_f1 then
        ClearOverrideBindings(followBtn)
        return
    end

    local leaderIdx = EazyBoxDB.leader_index or 1
    local leaderName = (EazyBoxDB.members[leaderIdx] or ""):trim()
    local myName = UnitName("player"):lower():trim()

    -- Si je ne suis pas le leader, F1 me fait suivre le leader automatiquement
    if leaderName ~= "" and leaderName:lower() ~= myName then
        followBtn:SetAttribute("macrotext", "/follow " .. leaderName)
        SetOverrideBindingClick(followBtn, true, "F1", "EazyBoxFollowButton")
    else
        -- Le leader ne se suit pas lui-meme
        followBtn:SetAttribute("macrotext", "")
        ClearOverrideBindings(followBtn)
    end
end

-- =========================================================================
-- 2. AUTO-ACCEPT DES INVITATIONS DU GROUPE
-- =========================================================================
local inviteFrame = CreateFrame("Frame", nil, UIParent)
inviteFrame:RegisterEvent("PARTY_INVITE_REQUEST")
inviteFrame:SetScript("OnEvent", function(self, event, inviter)
    if not (EazyBoxDB and EazyBoxDB.auto_invite) then return end
    if not inviter then return end

    local cleanInviter = inviter:match("([^-]+)") or inviter
    cleanInviter = cleanInviter:lower():trim()

    for _, member in ipairs(EazyBoxDB.members or {}) do
        local n = member:lower():trim()
        if n ~= "" and n == cleanInviter then
            AcceptGroup()
            StaticPopup_Hide("PARTY_INVITE")
            print("|cff00ddff[EazyBox]|r Invitation du groupe de |cff00ff00" .. inviter .. "|r acceptee automatiquement !")
            return
        end
    end
end)

-- =========================================================================
-- 3. INTERFACE PRINCIPALE DE CONFIGURATION (/eb)
-- =========================================================================
function EazyBox_InitGroupModule()
    mainFrame:SetSize(420, 480)
    mainFrame:SetPoint("CENTER", UIParent, "CENTER", 0, 80)
    mainFrame:SetBackdrop({
        bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background-Dark",
        edgeFile = "Interface\\Tooltips\\UI-Tooltip-Border",
        tile = true, tileSize = 16, edgeSize = 16,
        insets = { left = 4, right = 4, top = 4, bottom = 4 }
    })
    mainFrame:SetBackdropColor(0.06, 0.08, 0.12, 0.95)
    mainFrame:SetBackdropBorderColor(0, 0.8, 1, 0.8)
    mainFrame:SetMovable(true)
    mainFrame:EnableMouse(true)
    mainFrame:SetFrameStrata("HIGH")
    mainFrame:RegisterForDrag("LeftButton")
    mainFrame:SetScript("OnDragStart", mainFrame.StartMoving)
    mainFrame:SetScript("OnDragStop", mainFrame.StopMovingOrSizing)
    mainFrame:Hide()

    -- Titre
    local title = mainFrame:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    title:SetPoint("TOP", mainFrame, "TOP", 0, -12)
    title:SetText("|cff00ddff[EazyBox]|r - Multiboxing Controller")

    local sub = mainFrame:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    sub:SetPoint("TOP", title, "BOTTOM", 0, -4)
    sub:SetText("Gestion des 5 Comptes, Synchro & Voyants d'etat :")

    -- Bouton de fermeture
    local closeBtn = CreateFrame("Button", nil, mainFrame, "UIPanelButtonTemplate")
    closeBtn:SetSize(22, 22)
    closeBtn:SetPoint("TOPRIGHT", mainFrame, "TOPRIGHT", -8, -8)
    closeBtn:SetText("X")
    closeBtn:SetScript("OnClick", function() mainFrame:Hide() end)

    -- Formulaire pour les 5 membres avec voyants lumineux
    local labels = { "Slot 1 (Leader / Tank)", "Slot 2 (DPS 1)", "Slot 3 (DPS 2)", "Slot 4 (DPS 3)", "Slot 5 (Healer)" }

    for i = 1, 5 do
        local y = -50 - (i - 1) * 44

        -- Voyant lumineux (Texture circulaire avec statut vert / jaune / rouge)
        local dot = mainFrame:CreateTexture(nil, "OVERLAY")
        dot:SetSize(14, 14)
        dot:SetPoint("TOPLEFT", mainFrame, "TOPLEFT", 16, y - 18)
        dot:SetTexture("Interface\\RAIDFRAME\\Raid-Dot")
        dot:SetVertexColor(1, 0, 0) -- Rouge par defaut (hors-ligne)
        statusDots[i] = dot

        -- Label
        local lbl = mainFrame:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
        lbl:SetPoint("TOPLEFT", mainFrame, "TOPLEFT", 38, y)
        lbl:SetText(labels[i])

        -- Champ texte (EditBox)
        local eb = CreateFrame("EditBox", "EazyBoxMemberEdit" .. i, mainFrame, "InputBoxTemplate")
        eb:SetSize(190, 20)
        eb:SetPoint("TOPLEFT", lbl, "BOTTOMLEFT", 0, -2)
        eb:SetAutoFocus(false)
        eb:SetText(EazyBoxDB.members[i] or "")
        eb:SetScript("OnTextChanged", function(self)
            if EazyBoxDB and EazyBoxDB.members then
                EazyBoxDB.members[i] = self:GetText():trim()
                EazyBox_UpdateFollowBinding()
            end
        end)
        eb:SetScript("OnEnterPressed", function(self) self:ClearFocus() end)
        editBoxes[i] = eb

        -- Bouton de Role
        local rb = CreateFrame("Button", nil, mainFrame, "UIPanelButtonTemplate")
        rb:SetSize(72, 22)
        rb:SetPoint("LEFT", eb, "RIGHT", 8, 0)
        local currentRole = EazyBoxDB.roles[i] or "DPS"
        local rInfo = ROLE_INFO[currentRole] or ROLE_INFO.DPS
        rb:SetText(rInfo.color .. rInfo.label .. "|r")
        rb:SetScript("OnClick", function(self)
            local nextR = GetNextRole(EazyBoxDB.roles[i] or "DPS")
            EazyBoxDB.roles[i] = nextR
            local info = ROLE_INFO[nextR] or ROLE_INFO.DPS
            self:SetText(info.color .. info.label .. "|r")
            if EazyBox_UpdateHealGrid then EazyBox_UpdateHealGrid() end
        end)
        roleButtons[i] = rb
    end

    -- Bouton Auto-Remplir depuis le groupe
    local btnAutoFill = CreateFrame("Button", nil, mainFrame, "UIPanelButtonTemplate")
    btnAutoFill:SetSize(180, 24)
    btnAutoFill:SetPoint("TOPLEFT", mainFrame, "TOPLEFT", 18, -275)
    btnAutoFill:SetText("Remplir depuis le groupe")
    btnAutoFill:SetScript("OnClick", function()
        EazyBox_AutoFillFromParty()
    end)

    -- Bouton Inviter le groupe (5)
    local btnInvite = CreateFrame("Button", nil, mainFrame, "UIPanelButtonTemplate")
    btnInvite:SetSize(140, 24)
    btnInvite:SetPoint("LEFT", btnAutoFill, "RIGHT", 10, 0)
    btnInvite:SetText("Inviter Groupe (5)")
    btnInvite:SetScript("OnClick", function()
        EazyBox_InviteAllMembers()
    end)

    -- Options cases a cocher
    local function CreateCheckbox(labelText, yOffset, dbKey, onChange)
        local cb = CreateFrame("CheckButton", nil, mainFrame, "UICheckButtonTemplate")
        cb:SetPoint("TOPLEFT", mainFrame, "TOPLEFT", 18, yOffset)
        local cbLabel = cb:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
        cbLabel:SetPoint("LEFT", cb, "RIGHT", 4, 1)
        cbLabel:SetText(labelText)
        cb:SetChecked(EazyBoxDB[dbKey])
        cb:SetScript("OnClick", function(self)
            EazyBoxDB[dbKey] = self:GetChecked() and true or false
            if onChange then onChange(EazyBoxDB[dbKey]) end
        end)
        return cb
    end

    CreateCheckbox("|cff00ff88Auto-Quetes|r (Partager / Accepter / Rendre)", -310, "auto_quest")
    CreateCheckbox("|cff00ddffAuto-Invite|r (Accepter invitations du groupe)", -332, "auto_invite")
    CreateCheckbox("|cffffcc00Auto-Follow F1|r (Suivi sans macro manuelle)", -354, "auto_follow_f1", function(val)
        EazyBox_UpdateFollowBinding()
    end)
    CreateCheckbox("|cff44ff44Grille de Soins|r (Afficher l'interface de heal)", -376, "show_heal_grid", function(val)
        if EazyBoxHealFrame then
            if val then EazyBoxHealFrame:Show() else EazyBoxHealFrame:Hide() end
        end
    end)
    CreateCheckbox("|cff00ddffCarte & Radar HD (Carbonite)|r", -398, "show_map", function(val)
        if EazyBox_ToggleMap then
            EazyBox_ToggleMap(val)
        end
    end)

    -- Bouton Dissoudre
    local btnDisband = CreateFrame("Button", nil, mainFrame, "UIPanelButtonTemplate")
    btnDisband:SetSize(100, 24)
    btnDisband:SetPoint("BOTTOMLEFT", mainFrame, "BOTTOMLEFT", 18, 14)
    btnDisband:SetText("Dissoudre")
    btnDisband:SetScript("OnClick", function()
        LeaveParty()
        print("|cff00ddff[EazyBox]|r Groupe dissous.")
    end)

    -- Initialise l'auto-follow F1
    EazyBox_UpdateFollowBinding()
    EazyBox_UpdateGroupUI()
end

-- Met a jour les voyants lumineux et les noms de l'interface
function EazyBox_UpdateGroupUI()
    if not mainFrame:IsShown() then return end

    for i = 1, 5 do
        local dot = statusDots[i]
        local st = EazyBox_MemberStatus[i]

        if dot and st then
            if st.synced then
                -- Vert vif clignotant / allume : Connecte ET addon synchronise
                dot:SetVertexColor(0.1, 1.0, 0.1)
            elseif st.online then
                -- Jaune : Connecte dans le groupe mais addon en attente
                dot:SetVertexColor(1.0, 0.85, 0.1)
            else
                -- Rouge : Deconnecte / Hors-ligne
                dot:SetVertexColor(1.0, 0.15, 0.15)
            end
        end

        if editBoxes[i] and not editBoxes[i]:HasFocus() then
            editBoxes[i]:SetText(EazyBoxDB.members[i] or "")
        end
    end
end

-- Remplit automatiquement les champs depuis le groupe actuel
function EazyBox_AutoFillFromParty()
    local myName = UnitName("player")
    local list = { myName }

    for i = 1, GetNumPartyMembers() do
        local n = UnitName("party" .. i)
        if n and n ~= "" then
            table.insert(list, n)
        end
    end

    for i = 1, 5 do
        if list[i] then
            EazyBoxDB.members[i] = list[i]
            if editBoxes[i] then
                editBoxes[i]:SetText(list[i])
            end
        end
    end

    print("|cff00ddff[EazyBox]|r " .. #list .. " personnage(s) du groupe actuel enregistres dans l'addon !")
    EazyBox_UpdateFollowBinding()
    EazyBox_UpdatePresence()
    EazyBox_UpdateGroupUI()
end

-- Invite tous les membres configures
function EazyBox_InviteAllMembers()
    local myName = UnitName("player"):lower():trim()
    local count = 0
    for _, name in ipairs(EazyBoxDB.members or {}) do
        local n = name:trim()
        if n ~= "" and n:lower() ~= myName then
            InviteUnit(n)
            count = count + 1
        end
    end
    print("|cff00ddff[EazyBox]|r Invitations envoyees a " .. count .. " membres.")
end

function EazyBox_RefreshMainUI()
    EazyBox_UpdatePresence()
    EazyBox_UpdateGroupUI()
    EazyBox_UpdateFollowBinding()
end
