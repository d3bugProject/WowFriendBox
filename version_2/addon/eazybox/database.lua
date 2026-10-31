-- =========================================================================
-- EazyBox - database.lua
-- Base de donnees de ressources (Minerais & Herbes WotLK 3.3.5a)
-- Integration automatique avec CarboniteNodes si disponible
-- =========================================================================

EazyBox_NodesDB = {
    -- Definitions des types de ressources
    types = {
        MINE = {
            name = "Minage",
            color = { r = 1.0, g = 0.8, b = 0.0 }, -- Jaune/Or
            icon = "Interface\\Icons\\Trade_Mining",
        },
        HERB = {
            name = "Herboristerie",
            color = { r = 0.2, g = 1.0, b = 0.2 }, -- Vert vif
            icon = "Interface\\Icons\\Trade_Herbalism",
        },
    },

    -- Noms et categories de ressources
    resources = {
        -- WotLK & Burning Crusade Minerais
        ["Titane"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Titanium", color = {1.0, 0.4, 0.0} },
        ["Gisement de saronite riche"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Saronite", color = {0.8, 0.2, 1.0} },
        ["Gisement de saronite"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Saronite", color = {0.3, 0.6, 1.0} },
        ["Gisement de cobalt riche"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Cobalt", color = {0.2, 0.5, 1.0} },
        ["Gisement de cobalt"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Cobalt", color = {0.2, 0.8, 1.0} },
        ["Gisement de thorium riche"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Thorium", color = {0.4, 0.9, 0.7} },
        ["Filon de thorium"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Thorium", color = {0.5, 0.8, 0.6} },
        ["Filon de mithril"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Mithril", color = {0.6, 0.7, 0.9} },
        ["Filon de fer"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Iron", color = {0.7, 0.6, 0.5} },
        ["Filon d'etain"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Tin", color = {0.8, 0.5, 0.3} },
        ["Filon de cuivre"] = { type = "MINE", icon = "Interface\\Icons\\INV_Ore_Copper", color = {0.9, 0.6, 0.2} },

        -- WotLK & Burning Crusade Herbes
        ["Lotus givre"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_FrostLotus", color = {0.0, 1.0, 1.0} },
        ["Fleur-de-liche"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_Lichbloom", color = {0.3, 0.9, 1.0} },
        ["Glacepine"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_Icethorn", color = {0.6, 0.9, 1.0} },
        ["Glaurier"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_Icecap", color = {0.5, 1.0, 0.8} },
        ["Langue de serpent"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_AddersTongue", color = {0.3, 1.0, 0.4} },
        ["Rose de Talandra"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_TalandrasRose", color = {1.0, 0.3, 0.6} },
        ["Lys tigre"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_TigerLily", color = {1.0, 0.6, 0.2} },
        ["Trefle dore"] = { type = "HERB", icon = "Interface\\Icons\\INV_Herbalism_Goldclover", color = {1.0, 0.9, 0.2} },
        ["Fleur-de-peste"] = { type = "HERB", icon = "Interface\\Icons\\INV_Misc_Herb_Plaguebloom", color = {0.4, 0.8, 0.3} },
        ["Feuille-de-reve"] = { type = "HERB", icon = "Interface\\Icons\\INV_Misc_Herb_Dreamfoil", color = {0.2, 0.9, 0.5} },
        ["Sang-royal"] = { type = "HERB", icon = "Interface\\Icons\\INV_Misc_Herb_Kingsblood", color = {0.9, 0.2, 0.4} },
        ["Eglantine"] = { type = "HERB", icon = "Interface\\Icons\\INV_Misc_Herb_Briarthorn", color = {0.7, 0.5, 0.3} },
        ["Feuillargent"] = { type = "HERB", icon = "Interface\\Icons\\INV_Misc_Herb_Silverleaf", color = {0.8, 0.8, 0.9} },
        ["Paix-du-jour"] = { type = "HERB", icon = "Interface\\Icons\\INV_Misc_Herb_Peacebloom", color = {1.0, 1.0, 0.6} },
    },

    -- Base de donnees integree de coordonnees WotLK par nom de zone
    zones = {
        ["Bassin de Sholazar"] = {
            mapFile = "SholazarBasin",
            nodes = {
                { name = "Titane", x = 0.45, y = 0.32, type = "MINE" },
                { name = "Titane", x = 0.58, y = 0.41, type = "MINE" },
                { name = "Titane", x = 0.38, y = 0.65, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.28, y = 0.45, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.62, y = 0.28, type = "MINE" },
                { name = "Gisement de saronite", x = 0.35, y = 0.52, type = "MINE" },
                { name = "Gisement de saronite", x = 0.49, y = 0.62, type = "MINE" },
                { name = "Gisement de saronite", x = 0.54, y = 0.77, type = "MINE" },
                { name = "Gisement de saronite", x = 0.41, y = 0.25, type = "MINE" },
                { name = "Gisement de saronite", x = 0.67, y = 0.55, type = "MINE" },
                { name = "Langue de serpent", x = 0.48, y = 0.38, type = "HERB" },
                { name = "Langue de serpent", x = 0.52, y = 0.45, type = "HERB" },
                { name = "Langue de serpent", x = 0.41, y = 0.59, type = "HERB" },
                { name = "Lys tigre", x = 0.61, y = 0.49, type = "HERB" },
                { name = "Lys tigre", x = 0.33, y = 0.68, type = "HERB" },
                { name = "Lotus givre", x = 0.50, y = 0.40, type = "HERB" },
            }
        },
        ["Les Pics Foudroyés"] = {
            mapFile = "TheStormPeaks",
            nodes = {
                { name = "Titane", x = 0.37, y = 0.52, type = "MINE" },
                { name = "Titane", x = 0.44, y = 0.68, type = "MINE" },
                { name = "Titane", x = 0.62, y = 0.44, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.41, y = 0.58, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.55, y = 0.65, type = "MINE" },
                { name = "Gisement de saronite", x = 0.32, y = 0.45, type = "MINE" },
                { name = "Gisement de saronite", x = 0.49, y = 0.72, type = "MINE" },
                { name = "Fleur-de-liche", x = 0.38, y = 0.49, type = "HERB" },
                { name = "Fleur-de-liche", x = 0.45, y = 0.62, type = "HERB" },
                { name = "Glacepine", x = 0.52, y = 0.48, type = "HERB" },
                { name = "Glacepine", x = 0.60, y = 0.55, type = "HERB" },
                { name = "Lotus givre", x = 0.42, y = 0.55, type = "HERB" },
            }
        },
        ["La Couronne de glace"] = {
            mapFile = "Icecrown",
            nodes = {
                { name = "Titane", x = 0.55, y = 0.38, type = "MINE" },
                { name = "Titane", x = 0.48, y = 0.72, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.62, y = 0.42, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.41, y = 0.66, type = "MINE" },
                { name = "Gisement de saronite", x = 0.53, y = 0.52, type = "MINE" },
                { name = "Gisement de saronite", x = 0.35, y = 0.48, type = "MINE" },
                { name = "Fleur-de-liche", x = 0.59, y = 0.35, type = "HERB" },
                { name = "Glacepine", x = 0.45, y = 0.58, type = "HERB" },
                { name = "Glacepine", x = 0.65, y = 0.45, type = "HERB" },
                { name = "Lotus givre", x = 0.51, y = 0.50, type = "HERB" },
            }
        },
        ["Toundra Boréenne"] = {
            mapFile = "BoreanTundra",
            nodes = {
                { name = "Gisement de cobalt riche", x = 0.45, y = 0.33, type = "MINE" },
                { name = "Gisement de cobalt", x = 0.55, y = 0.42, type = "MINE" },
                { name = "Gisement de cobalt", x = 0.38, y = 0.52, type = "MINE" },
                { name = "Gisement de cobalt", x = 0.62, y = 0.65, type = "MINE" },
                { name = "Trefle dore", x = 0.52, y = 0.38, type = "HERB" },
                { name = "Trefle dore", x = 0.48, y = 0.55, type = "HERB" },
                { name = "Rose de Talandra", x = 0.42, y = 0.62, type = "HERB" },
            }
        },
        ["Fjord Hurlant"] = {
            mapFile = "HowlingFjord",
            nodes = {
                { name = "Gisement de cobalt riche", x = 0.52, y = 0.35, type = "MINE" },
                { name = "Gisement de cobalt", x = 0.48, y = 0.58, type = "MINE" },
                { name = "Gisement de cobalt", x = 0.60, y = 0.45, type = "MINE" },
                { name = "Trefle dore", x = 0.55, y = 0.42, type = "HERB" },
                { name = "Lys tigre", x = 0.62, y = 0.68, type = "HERB" },
            }
        },
        ["Les Grisonnes"] = {
            mapFile = "GrizzlyHills",
            nodes = {
                { name = "Gisement de cobalt riche", x = 0.42, y = 0.48, type = "MINE" },
                { name = "Gisement de cobalt", x = 0.58, y = 0.36, type = "MINE" },
                { name = "Gisement de saronite", x = 0.65, y = 0.72, type = "MINE" },
                { name = "Trefle dore", x = 0.45, y = 0.52, type = "HERB" },
                { name = "Lys tigre", x = 0.38, y = 0.60, type = "HERB" },
            }
        },
        ["Joug-d'hiver"] = {
            mapFile = "LakeWintergrasp",
            nodes = {
                { name = "Titane", x = 0.45, y = 0.28, type = "MINE" },
                { name = "Titane", x = 0.60, y = 0.52, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.38, y = 0.45, type = "MINE" },
                { name = "Gisement de saronite", x = 0.52, y = 0.65, type = "MINE" },
                { name = "Glacepine", x = 0.40, y = 0.55, type = "HERB" },
                { name = "Fleur-de-liche", x = 0.55, y = 0.42, type = "HERB" },
                { name = "Lotus givre", x = 0.50, y = 0.50, type = "HERB" },
            }
        },
        ["Dalaran"] = {
            mapFile = "Dalaran",
            nodes = {
                { name = "Titane", x = 0.48, y = 0.42, type = "MINE" },
                { name = "Gisement de saronite riche", x = 0.55, y = 0.58, type = "MINE" },
                { name = "Lotus givre", x = 0.42, y = 0.62, type = "HERB" },
                { name = "Fleur-de-liche", x = 0.58, y = 0.38, type = "HERB" },
            }
        },
        ["Durotar"] = {
            mapFile = "Durotar",
            nodes = {
                { name = "Filon de cuivre", x = 0.45, y = 0.25, type = "MINE" },
                { name = "Filon de cuivre", x = 0.52, y = 0.42, type = "MINE" },
                { name = "Filon de cuivre", x = 0.41, y = 0.58, type = "MINE" },
                { name = "Filon de cuivre", x = 0.55, y = 0.72, type = "MINE" },
                { name = "Filon d'etain", x = 0.42, y = 0.18, type = "MINE" },
                { name = "Paix-du-jour", x = 0.48, y = 0.35, type = "HERB" },
                { name = "Feuillargent", x = 0.51, y = 0.48, type = "HERB" },
                { name = "Feuillargent", x = 0.43, y = 0.65, type = "HERB" },
            }
        },
        ["Forêt d'Elwynn"] = {
            mapFile = "Elwynn",
            nodes = {
                { name = "Filon de cuivre", x = 0.42, y = 0.55, type = "MINE" },
                { name = "Filon de cuivre", x = 0.55, y = 0.62, type = "MINE" },
                { name = "Filon de cuivre", x = 0.38, y = 0.75, type = "MINE" },
                { name = "Paix-du-jour", x = 0.45, y = 0.48, type = "HERB" },
                { name = "Feuillargent", x = 0.52, y = 0.58, type = "HERB" },
            }
        },
        ["Les Tarides"] = {
            mapFile = "Barrens",
            nodes = {
                { name = "Filon de cuivre", x = 0.52, y = 0.35, type = "MINE" },
                { name = "Filon d'etain", x = 0.48, y = 0.45, type = "MINE" },
                { name = "Filon de fer", x = 0.44, y = 0.60, type = "MINE" },
                { name = "Eglantine", x = 0.50, y = 0.38, type = "HERB" },
                { name = "Sang-royal", x = 0.46, y = 0.55, type = "HERB" },
            }
        },
        ["Désolation des dragons"] = {
            mapFile = "Dragonblight",
            nodes = {
                { name = "Gisement de cobalt", x = 0.48, y = 0.35, type = "MINE" },
                { name = "Gisement de cobalt riche", x = 0.58, y = 0.52, type = "MINE" },
                { name = "Trefle dore", x = 0.52, y = 0.42, type = "HERB" },
                { name = "Lotus givre", x = 0.45, y = 0.65, type = "HERB" },
            }
        },
    }
}

-- Fonction pour recuperer la liste des nodes pour une zone donnee
function EazyBox_GetZoneNodes(zoneName, mapFile)
    if not zoneName and not mapFile then return {} end
    local lowerZone = (zoneName or ""):lower():trim()
    local lowerMap = (mapFile or ""):lower():trim()

    for zKey, data in pairs(EazyBox_NodesDB.zones) do
        local lKey = zKey:lower():trim()
        local lMap = (data.mapFile or ""):lower():trim()
        if (lowerZone ~= "" and (lowerZone == lKey or lowerZone:find(lKey) or lKey:find(lowerZone)))
           or (lowerMap ~= "" and (lowerMap == lMap or lowerMap:find(lMap) or lMap:find(lowerMap))) then
            return data.nodes or {}
        end
    end
    return {}
end

-- Fonction d'import dynamique depuis CarboniteNodes si l'addon est charge
function EazyBox_CheckCarboniteIntegration()
    if CarboniteNodes and CarboniteNodes.NXHerb then
        return true
    end
    return false
end
