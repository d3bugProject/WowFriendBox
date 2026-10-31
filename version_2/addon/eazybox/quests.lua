-- =========================================================================
-- EazyBox - quests.lua
-- Automatisation complete des quetes pour le multiboxing :
-- Acceptation instantanee, Partage automatique au groupe, Validation,
-- et Choix de la recompense ayant la plus haute valeur de revente (PO).
-- =========================================================================

local questFrame = CreateFrame("Frame", "EazyBoxQuestFrame", UIParent)

function EazyBox_InitQuestsModule()
    questFrame:RegisterEvent("QUEST_DETAIL")
    questFrame:RegisterEvent("QUEST_PROGRESS")
    questFrame:RegisterEvent("QUEST_COMPLETE")
    questFrame:RegisterEvent("QUEST_ACCEPT_CONFIRM")
    questFrame:RegisterEvent("QUEST_GREET")
    questFrame:RegisterEvent("GOSSIP_SHOW")
    questFrame:RegisterEvent("UI_INFO_MESSAGE")

    -- Hook StaticPopup pour eviter les popups de validation
    hooksecurefunc("StaticPopup_Show", function(which)
        if not (EazyBoxDB and EazyBoxDB.auto_quest) then return end
        if which == "QUEST_ACCEPT" then
            AcceptQuest()
            StaticPopup_Hide("QUEST_ACCEPT")
        elseif which == "QUEST_ACCEPT_CONFIRM" then
            ConfirmAcceptQuest()
            StaticPopup_Hide("QUEST_ACCEPT_CONFIRM")
        end
    end)
end

questFrame:SetScript("OnEvent", function(self, event, ...)
    if not (EazyBoxDB and EazyBoxDB.auto_quest) then return end

    -- 1. Acceptation immediate d'une quete proposee par un PNJ
    if event == "QUEST_DETAIL" then
        if not QuestGetAutoAccept() then
            AcceptQuest()
            print("|cff00ddff[EazyBox]|r Quete acceptee automatiquement.")
        end

    -- 2. Quete d'escorte ou partagee avec confirmation
    elseif event == "QUEST_ACCEPT_CONFIRM" then
        ConfirmAcceptQuest()
        StaticPopup_Hide("QUEST_ACCEPT_CONFIRM")

    -- 3. Liste de quetes chez un PNJ (QuestGreet)
    elseif event == "QUEST_GREET" then
        local numActive = GetNumActiveQuests()
        local numAvailable = GetNumAvailableQuests()

        -- S'il y a des quetes pretes a etre rendues, on ouvre la premiere
        if numActive > 0 then
            for i = 1, numActive do
                local title, isComplete = GetActiveTitle(i)
                if isComplete then
                    SelectActiveQuest(i)
                    return
                end
            end
        end

        -- S'il n'y a qu'une seule quete disponible, on l'ouvre
        if numAvailable == 1 and numActive == 0 then
            SelectAvailableQuest(1)
        end

    -- 4. Dialogue Gossip avec un PNJ
    elseif event == "GOSSIP_SHOW" then
        if not IsShiftKeyDown() then
            local numActive = GetNumGossipActiveQuests()
            local numAvailable = GetNumGossipAvailableQuests()

            -- Rend les quetes terminees en priorite
            if numActive > 0 then
                for i = 1, numActive do
                    local _, _, _, isComplete = GetGossipActiveQuests()
                    if isComplete then
                        SelectGossipActiveQuest(i)
                        return
                    end
                end
            end

            -- Si une seule quete disponible : ouverture automatique
            if numAvailable == 1 and numActive == 0 then
                SelectGossipAvailableQuest(1)
            end
        end

    -- 5. Progression de quete (Page ou le PNJ demande si on a fini)
    elseif event == "QUEST_PROGRESS" then
        if IsQuestCompletable() then
            CompleteQuest()
        end

    -- 6. Rendu de quete & Choix intelligent de la recompense la plus chere
    elseif event == "QUEST_COMPLETE" then
        local numChoices = GetNumQuestChoices()
        if numChoices > 1 then
            local bestIndex = 1
            local bestPrice = 0

            for i = 1, numChoices do
                local link = GetQuestItemLink("choice", i)
                if link then
                    local _, _, _, _, _, _, _, _, _, _, vendorPrice = GetItemInfo(link)
                    if vendorPrice and vendorPrice > bestPrice then
                        bestPrice = vendorPrice
                        bestIndex = i
                    end
                end
            end

            GetQuestReward(bestIndex)
            local gold = math.floor(bestPrice / 10000)
            local silver = math.floor((bestPrice % 10000) / 100)
            print(string.format("|cff00ddff[EazyBox]|r Quete validee ! Recompense optimale choisie (%d po %d pa).", gold, silver))
        else
            GetQuestReward(1)
            print("|cff00ddff[EazyBox]|r Quete validee et rendue !")
        end
    end
end)
