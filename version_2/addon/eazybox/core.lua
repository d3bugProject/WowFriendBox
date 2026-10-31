-- =========================================================================
-- EazyBox - core.lua
-- Initialisation, communication reseau en groupe, battement de coeur (Heartbeat),
-- commandes slash et coordination des modules.
-- =========================================================================

local ADDON_NAME = "eazybox"
local ADDON_PREFIX = "EAZYBOX_SYNC"
local VERSION = "2.0.0"

-- Base de donnees par defaut
local defaultDB = {
    version = VERSION,
    members = { "mel7", "goldrogerz", "elewel", "ezawedj", "etalet" },
    roles = { "TANK", "DPS", "DPS", "DPS", "HEAL" },
    leader_index = 1,
    auto_invite = true,
    auto_quest = true,
    auto_follow_f1 = true,
    auto_taxi = true,
    show_heal_grid = true,
    lock_heal_grid = false,
    heal_grid_pos = { x = 20, y = 280 },
    show_map = true,
    lock_map = false,
    map_pos = { x = 500, y = 300, w = 260, h = 260 },
    map_zoom = 1.0,
    map_show_mines = true,
    map_show_herbs = true,
    radar_pulse_enabled = true,
}

-- Table globale de statut en temps reel pour les 5 membres
EazyBox_MemberStatus = {}
for i = 1, 5 do
    EazyBox_MemberStatus[i] = {
        name = "",
        online = false,
        synced = false,
        lastSeen = 0,
        hp = 100,
        maxHp = 100,
        mana = 100,
        maxMana = 100,
        zone = "",
    }
end

-- Cadre principal d'evenements
local eventFrame = CreateFrame("Frame", "EazyBoxEventFrame", UIParent)

-- Initialisation a la connexion
eventFrame:RegisterEvent("ADDON_LOADED")
eventFrame:RegisterEvent("PLAYER_LOGIN")
eventFrame:RegisterEvent("CHAT_MSG_ADDON")
eventFrame:RegisterEvent("PARTY_MEMBERS_CHANGED")
eventFrame:RegisterEvent("PLAYER_ENTERING_WORLD")

eventFrame:SetScript("OnEvent", function(self, event, ...)
    if event == "ADDON_LOADED" then
        local name = ...
        if name == ADDON_NAME then
            if not EazyBoxDB then
                EazyBoxDB = defaultDB
            else
                -- Migration et fusion des champs manquants
                for k, v in pairs(defaultDB) do
                    if EazyBoxDB[k] == nil then
                        EazyBoxDB[k] = v
                    end
                end
            end
        end

    elseif event == "PLAYER_LOGIN" then
        print("|cff00ddff[EazyBox v" .. VERSION .. "]|r Multibox Manager charge avec succes ! Tapez |cff00ff00/eb|r pour le menu.")
        
        -- Enregistre le prefixe de communication addon si supporte par la version du client
        if RegisterAddonMessagePrefix then
            RegisterAddonMessagePrefix(ADDON_PREFIX)
        end

        -- Initialise les interfaces
        if EazyBox_InitGroupModule then EazyBox_InitGroupModule() end
        if EazyBox_InitQuestsModule then EazyBox_InitQuestsModule() end
        if EazyBox_InitHealModule then EazyBox_InitHealModule() end
        if EazyBox_InitMapModule then EazyBox_InitMapModule() end

    elseif event == "CHAT_MSG_ADDON" then
        local prefix, message, channel, sender = ...
        if prefix == ADDON_PREFIX then
            EazyBox_HandleAddonMessage(message, sender)
        end

    elseif event == "PARTY_MEMBERS_CHANGED" or event == "PLAYER_ENTERING_WORLD" then
        EazyBox_UpdatePresence()
        if EazyBox_UpdateHealGrid then EazyBox_UpdateHealGrid() end
        if EazyBox_UpdateGroupUI then EazyBox_UpdateGroupUI() end
        if EazyBox_UpdateFollowBinding then EazyBox_UpdateFollowBinding() end
    end
end)

-- Reception des messages de synchronisation
function EazyBox_HandleAddonMessage(message, sender)
    if not sender then return end
    local cleanSender = sender:match("([^-]+)") or sender
    cleanSender = cleanSender:lower():trim()

    local parts = { strsplit(":", message) }
    local cmd = parts[1]

    if cmd == "PING" or cmd == "STATUS" then
        -- Format STATUS:hp:maxHp:mana:maxMana:zone
        local hp = tonumber(parts[2]) or 100
        local maxHp = tonumber(parts[3]) or 100
        local mana = tonumber(parts[4]) or 100
        local maxMana = tonumber(parts[5]) or 100
        local zone = parts[6] or ""

        -- Met a jour le slot correspondant
        for i = 1, 5 do
            local cfgName = (EazyBoxDB and EazyBoxDB.members[i] or ""):lower():trim()
            if cfgName ~= "" and cfgName == cleanSender then
                EazyBox_MemberStatus[i].name = cfgName
                EazyBox_MemberStatus[i].online = true
                EazyBox_MemberStatus[i].synced = true
                EazyBox_MemberStatus[i].lastSeen = GetTime()
                EazyBox_MemberStatus[i].hp = hp
                EazyBox_MemberStatus[i].maxHp = maxHp
                EazyBox_MemberStatus[i].mana = mana
                EazyBox_MemberStatus[i].maxMana = maxMana
                EazyBox_MemberStatus[i].zone = zone
                break
            end
        end

        if EazyBox_UpdateGroupUI then EazyBox_UpdateGroupUI() end
        if EazyBox_UpdateHealGrid then EazyBox_UpdateHealGrid() end

    elseif cmd == "FOLLOW_REQ" then
        -- Demande de suivi recue
        local leader = parts[2]
        if leader and leader ~= "" then
            FollowUnit(leader)
        end
    end
end

-- Verification reguliere de la presence (Heartbeat toutes les 2.5 secondes)
local heartbeatTimer = 0
local checkTimer = 0

local tickerFrame = CreateFrame("Frame", nil, UIParent)
tickerFrame:SetScript("OnUpdate", function(self, elapsed)
    heartbeatTimer = heartbeatTimer + elapsed
    checkTimer = checkTimer + elapsed

    -- 1. Envoi du Heartbeat
    if heartbeatTimer >= 2.5 then
        heartbeatTimer = 0
        if GetNumPartyMembers() > 0 or GetNumRaidMembers() > 0 then
            local hp = UnitHealth("player") or 100
            local maxHp = UnitHealthMax("player") or 100
            local mana = UnitPower("player") or 100
            local maxMana = UnitPowerMax("player") or 100
            local zone = GetZoneText() or ""

            local payload = string.format("STATUS:%d:%d:%d:%d:%s", hp, maxHp, mana, maxMana, zone)
            SendAddonMessage(ADDON_PREFIX, payload, "PARTY")
        end
    end

    -- 2. Verification de la date de reception (expiration > 6s = non synchro)
    if checkTimer >= 1.5 then
        checkTimer = 0
        local now = GetTime()
        local myName = UnitName("player"):lower():trim()

        for i = 1, 5 do
            local cfgName = (EazyBoxDB and EazyBoxDB.members[i] or ""):lower():trim()
            if cfgName ~= "" then
                if cfgName == myName then
                    EazyBox_MemberStatus[i].name = cfgName
                    EazyBox_MemberStatus[i].online = true
                    EazyBox_MemberStatus[i].synced = true
                    EazyBox_MemberStatus[i].hp = UnitHealth("player")
                    EazyBox_MemberStatus[i].maxHp = UnitHealthMax("player")
                    EazyBox_MemberStatus[i].mana = UnitPower("player")
                    EazyBox_MemberStatus[i].maxMana = UnitPowerMax("player")
                else
                    -- Verifie si l'unite est dans le groupe WoW
                    local unit = EazyBox_GetUnitForName(cfgName)
                    local isUnitConnected = unit and UnitIsConnected(unit)

                    if now - EazyBox_MemberStatus[i].lastSeen > 7.0 then
                        EazyBox_MemberStatus[i].synced = false
                    end

                    EazyBox_MemberStatus[i].online = isUnitConnected and true or false
                end
            else
                EazyBox_MemberStatus[i].online = false
                EazyBox_MemberStatus[i].synced = false
            end
        end

        if EazyBox_UpdateGroupUI then EazyBox_UpdateGroupUI() end
    end
end)

-- Met a jour la presence en fonction des membres du groupe WoW
function EazyBox_UpdatePresence()
    local myName = UnitName("player"):lower():trim()
    for i = 1, 5 do
        local cfgName = (EazyBoxDB and EazyBoxDB.members[i] or ""):lower():trim()
        if cfgName == myName then
            EazyBox_MemberStatus[i].online = true
            EazyBox_MemberStatus[i].synced = true
        else
            local unit = EazyBox_GetUnitForName(cfgName)
            if unit and UnitIsConnected(unit) then
                EazyBox_MemberStatus[i].online = true
            else
                EazyBox_MemberStatus[i].online = false
                EazyBox_MemberStatus[i].synced = false
            end
        end
    end
end

-- Helper pour retrouver l'ID d'unite ("party1".."party4" ou "player") a partir du nom
function EazyBox_GetUnitForName(targetName)
    if not targetName or targetName == "" then return nil end
    local lower = targetName:lower():trim()
    if UnitName("player"):lower():trim() == lower then
        return "player"
    end
    for i = 1, GetNumPartyMembers() do
        local u = "party" .. i
        local n = UnitName(u)
        if n and n:lower():trim() == lower then
            return u
        end
    end
    return nil
end

-- =========================================================================
-- COMMANDES SLASH (/eb, /eazybox, /eazy)
-- =========================================================================
SLASH_EAZYBOX1 = "/eb"
SLASH_EAZYBOX2 = "/eazybox"
SLASH_EAZYBOX3 = "/eazy"

SlashCmdList["EAZYBOX"] = function(msg)
    local arg = (msg or ""):lower():trim()
    if arg == "inv" or arg == "invite" then
        if EazyBox_InviteAllMembers then EazyBox_InviteAllMembers() end
    elseif arg == "dis" or arg == "disband" then
        LeaveParty()
        print("|cff00ddff[EazyBox]|r Groupe dissous.")
    elseif arg == "heal" or arg == "grid" then
        if EazyBoxHealFrame then
            if EazyBoxHealFrame:IsShown() then
                EazyBoxHealFrame:Hide()
                EazyBoxDB.show_heal_grid = false
            else
                EazyBoxHealFrame:Show()
                EazyBoxDB.show_heal_grid = true
            end
        end
    elseif arg == "map" or arg == "radar" or arg == "carb" or arg == "carbonite" then
        if EazyBox_ToggleMap then
            EazyBox_ToggleMap()
        end
    elseif arg == "import" or arg == "nodes" then
        if EazyBox_AutoImportNodes then
            EazyBox_AutoImportNodes(true)
        end
    else
        -- Ouvre ou ferme la fenetre principale
        if EazyBoxMainFrame then
            if EazyBoxMainFrame:IsShown() then
                EazyBoxMainFrame:Hide()
            else
                EazyBoxMainFrame:Show()
                if EazyBox_UpdateGroupUI then EazyBox_UpdateGroupUI() end
            end
        end
    end
end
