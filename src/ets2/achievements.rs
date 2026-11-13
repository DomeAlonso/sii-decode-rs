use std::collections::{BTreeMap, BTreeSet};

use crate::ets2::achievement_definitions::{ACHIEVEMENTS, ACHIEVEMENT_GROUPS};
use crate::ets2::generated::cargo_metadata::CARGOS;
use crate::ets2::save::{
    cargo_id, company_city, company_id, truck_brand, DeliveryLogEntry, SaveGame,
};
use crate::ets2::CargoMetadata;

pub const EXPERIENCE_BEATS_ALL_CATEGORIES: [&str; 8] = [
    "Machinery",
    "ADR cargo",
    "Container",
    "Refrigerated",
    "Liquid cargo",
    "Fragile cargo",
    "Construction",
    "Bulk cargo",
];

const TEST_DRIVE_LIMITED_TARGET_BRANDS: u32 = 5;
const TEST_DRIVE_LIMITED_TARGET_DISTANCE_KM: u64 = 999;
const OWNED_TRUCK_JOB_TYPES: [&str; 4] = ["cargo", "external", "compn", "on_compn"];
const ALL_IS_POSSIBLE_TARGET_CARGOS: u32 = 30;
const RELIABLE_CONTRACTOR_TARGET_COMPANIES: u32 = 15;
const LONG_HAULER_TARGET_DISTANCE_KM: u32 = 2000;
const PROFIT_HUNTER_TARGET_REVENUE: f64 = 130_000.0;
const PROFIT_HUNTER_TARGET_DISTANCE_KM: u32 = 2200;
const SUMMARY_EVIDENCE_LIMIT: usize = 30;

/// `job_type` of Special Transport (oversize) jobs in the delivery log.
const OVERSIZE_JOB_TYPE: &str = "spec_oversize";
const THE_BIGGER_THE_BETTER_TARGET_CARGOS: u32 = 10;
const TURKISH_DELIGHT_TARGET_DELIVERIES: u32 = 3;
const TURKISH_DELIGHT_MIN_DISTANCE_KM: u32 = 2500;

// City IDs below follow the game's `company.volatile.<company>.<city>` naming,
// which uses local spellings (e.g. `munchen`, `wien`, `bucuresti`).
const ISTANBUL: &str = "istanbul";
const ORIENT_EXPRESS_LEGS: [(&str, &str); 6] = [
    ("paris", "strasbourg"),
    ("strasbourg", "munchen"),
    ("munchen", "wien"),
    ("wien", "budapest"),
    ("budapest", "bucuresti"),
    ("bucuresti", ISTANBUL),
];
const IBERIAN_PILGRIMAGE_LEGS: [(&str, &str); 3] = [
    ("lisboa", "a_coruna"),
    ("sevilla", "a_coruna"),
    ("pamplona", "a_coruna"),
];
const I_AM_A_GPS_TARGET_KM: u32 = 20_000;
const PATHFINDER_TARGET_KM: u32 = 40_000;
const HEAD_HUNTER_TARGET_AGENCIES: u32 = 40;
const PROPERTY_MAGNATE_TARGET_GARAGES: u32 = 30;
// Company/city IDs below were checked against a 1.5x save's company list.
const FRENCH_NUCLEAR_PLANT_CITIES: [&str; 5] = ["alban", "civaux", "golfech", "laurent", "paluel"];
const SCANDINAVIA_CONTAINER_PORT_CITIES: [&str; 2] = ["esbjerg", "oslo"];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementRegistry {
    pub groups: Vec<AchievementGroup>,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Achievement {
    pub id: &'static str,
    pub group: &'static str,
    pub steam_api_name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub status: AchievementStatus,
    pub progress: AchievementProgress,
    pub evidence: Vec<AchievementEvidence>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AchievementStatus {
    Complete,
    InProgress,
    /// The save data this achievement depends on is not read by the tracker yet.
    NotTracked,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementProgress {
    pub current: u32,
    pub target: u32,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementEvidence {
    pub label: String,
    pub value: String,
    pub items: Vec<String>,
    pub complete: bool,
}

/// One Steam achievement set: the base game, a DLC, or another special set.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AchievementGroup {
    pub id: &'static str,
    pub name: &'static str,
    /// The set's `id` in Steam's achievement schema.
    pub steam_group_id: u32,
    /// The DLC's Steam app id, when the set belongs to one DLC.
    pub steam_dlc_app_id: Option<u32>,
}

/// Static description of one Steam achievement and how the tracker evaluates it.
#[derive(Debug, Clone, Copy)]
pub struct AchievementDefinition {
    pub id: &'static str,
    /// The `id` of the [`AchievementGroup`] this achievement belongs to.
    pub group: &'static str,
    /// The achievement's `api_name` on Steam (app 227300).
    pub steam_api_name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub tracking: AchievementTracking,
}

#[derive(Debug, Clone, Copy)]
pub enum AchievementTracking {
    /// Evaluated from the save game's delivery log.
    DeliveryLog(fn(&SaveGame) -> AchievementEvaluation),
    /// Listed for completeness; `reason` says what the tracker is missing.
    NotTracked { reason: &'static str },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementEvaluation {
    pub status: AchievementStatus,
    pub progress: AchievementProgress,
    pub evidence: Vec<AchievementEvidence>,
}

pub fn evaluate_achievements(save: &SaveGame) -> AchievementRegistry {
    AchievementRegistry {
        groups: ACHIEVEMENT_GROUPS.to_vec(),
        achievements: ACHIEVEMENTS
            .iter()
            .map(|definition| evaluate_definition(definition, save))
            .collect(),
    }
}

fn evaluate_definition(definition: &AchievementDefinition, save: &SaveGame) -> Achievement {
    let evaluation = match definition.tracking {
        AchievementTracking::DeliveryLog(evaluate) => evaluate(save),
        AchievementTracking::NotTracked { reason } => AchievementEvaluation {
            status: AchievementStatus::NotTracked,
            progress: AchievementProgress {
                current: 0,
                target: 0,
                unit: "",
            },
            evidence: vec![AchievementEvidence {
                label: "Not tracked yet".to_string(),
                value: reason.to_string(),
                items: Vec::new(),
                complete: false,
            }],
        },
    };

    Achievement {
        id: definition.id,
        group: definition.group,
        steam_api_name: definition.steam_api_name,
        display_name: definition.display_name,
        description: definition.description,
        status: evaluation.status,
        progress: evaluation.progress,
        evidence: evaluation.evidence,
    }
}

pub(crate) fn experience_beats_all(save: &SaveGame) -> AchievementEvaluation {
    let mut evidence_by_category = EXPERIENCE_BEATS_ALL_CATEGORIES
        .into_iter()
        .map(|category| (category, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for entry in &save.delivery_log.entries {
        for category in achievement_categories_for_cargo(&entry.cargo) {
            if let Some(cargos) = evidence_by_category.get_mut(category) {
                cargos.insert(cargo_id(&entry.cargo).to_string());
            }
        }
    }

    let evidence = EXPERIENCE_BEATS_ALL_CATEGORIES
        .into_iter()
        .map(|category| {
            let cargos = &evidence_by_category[category];
            AchievementEvidence {
                label: category.to_string(),
                value: summarize_set(cargos, SUMMARY_EVIDENCE_LIMIT),
                items: set_items(cargos),
                complete: !cargos.is_empty(),
            }
        })
        .collect::<Vec<_>>();
    let current = evidence.iter().filter(|item| item.complete).count() as u32;
    let target = EXPERIENCE_BEATS_ALL_CATEGORIES.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, target),
        progress: AchievementProgress {
            current,
            target,
            unit: "categories",
        },
        evidence,
    }
}

pub(crate) fn test_drive_limited(save: &SaveGame) -> AchievementEvaluation {
    let mut distance_by_brand = BTreeMap::<String, u64>::new();
    for entry in save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| is_owned_truck_job_type(&entry.job_type))
    {
        *distance_by_brand
            .entry(display_truck_brand(truck_brand(&entry.truck)).to_string())
            .or_insert(0) += u64::from(entry.distance_km);
    }

    let mut brands = distance_by_brand.into_iter().collect::<Vec<_>>();
    brands.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let evidence = brands
        .iter()
        .map(|(brand, distance)| AchievementEvidence {
            label: brand.clone(),
            value: format!("{distance} km"),
            items: Vec::new(),
            complete: *distance >= TEST_DRIVE_LIMITED_TARGET_DISTANCE_KM,
        })
        .collect::<Vec<_>>();
    let current = evidence.iter().filter(|item| item.complete).count() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, TEST_DRIVE_LIMITED_TARGET_BRANDS),
        progress: AchievementProgress {
            current,
            target: TEST_DRIVE_LIMITED_TARGET_BRANDS,
            unit: "brands",
        },
        evidence,
    }
}

pub(crate) fn all_is_possible(save: &SaveGame) -> AchievementEvaluation {
    let cargos = save
        .delivery_log
        .unique_cargos()
        .into_iter()
        .filter(|cargo| !cargo.is_empty())
        .collect::<BTreeSet<_>>();
    let current = cargos.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, ALL_IS_POSSIBLE_TARGET_CARGOS),
        progress: AchievementProgress {
            current,
            target: ALL_IS_POSSIBLE_TARGET_CARGOS,
            unit: "cargoes",
        },
        evidence: vec![AchievementEvidence {
            label: "Completed cargoes".to_string(),
            value: summarize_set(&cargos, SUMMARY_EVIDENCE_LIMIT),
            items: set_items(&cargos),
            complete: current >= ALL_IS_POSSIBLE_TARGET_CARGOS,
        }],
    }
}

pub(crate) fn reliable_contractor(save: &SaveGame) -> AchievementEvaluation {
    let companies = save
        .delivery_log
        .entries
        .iter()
        .flat_map(|entry| {
            [
                company_id(&entry.source_company).to_string(),
                company_id(&entry.destination_company).to_string(),
            ]
        })
        .filter(|company| !company.is_empty())
        .collect::<BTreeSet<_>>();
    let current = companies.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, RELIABLE_CONTRACTOR_TARGET_COMPANIES),
        progress: AchievementProgress {
            current,
            target: RELIABLE_CONTRACTOR_TARGET_COMPANIES,
            unit: "companies",
        },
        evidence: vec![AchievementEvidence {
            label: "Companies".to_string(),
            value: summarize_set(&companies, SUMMARY_EVIDENCE_LIMIT),
            items: set_items(&companies),
            complete: current >= RELIABLE_CONTRACTOR_TARGET_COMPANIES,
        }],
    }
}

pub(crate) fn long_hauler(save: &SaveGame) -> AchievementEvaluation {
    let longest = save
        .delivery_log
        .entries
        .iter()
        .max_by_key(|entry| entry.distance_km);
    let current = longest.map_or(0, |entry| entry.distance_km);
    let complete = current > LONG_HAULER_TARGET_DISTANCE_KM;

    AchievementEvaluation {
        status: if complete {
            AchievementStatus::Complete
        } else {
            AchievementStatus::InProgress
        },
        progress: AchievementProgress {
            current,
            target: LONG_HAULER_TARGET_DISTANCE_KM,
            unit: "km",
        },
        evidence: vec![AchievementEvidence {
            label: "Longest delivery".to_string(),
            value: longest
                .map(delivery_summary)
                .unwrap_or_else(|| "missing".to_string()),
            items: Vec::new(),
            complete,
        }],
    }
}

pub(crate) fn profit_hunter(save: &SaveGame) -> AchievementEvaluation {
    let qualifying = save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| is_profit_hunter_delivery(entry))
        .max_by(|left, right| left.revenue.total_cmp(&right.revenue));
    let evidence_entry = qualifying.or_else(|| {
        save.delivery_log
            .entries
            .iter()
            .max_by(|left, right| left.revenue.total_cmp(&right.revenue))
    });
    let current = evidence_entry.map_or(0, profit_hunter_criteria_met);
    let complete = qualifying.is_some();

    AchievementEvaluation {
        status: if complete {
            AchievementStatus::Complete
        } else {
            AchievementStatus::InProgress
        },
        progress: AchievementProgress {
            current,
            target: 2,
            unit: "requirements",
        },
        evidence: vec![AchievementEvidence {
            label: if complete {
                "Qualifying job".to_string()
            } else {
                "Best revenue job".to_string()
            },
            value: evidence_entry
                .map(delivery_summary)
                .unwrap_or_else(|| "missing".to_string()),
            items: Vec::new(),
            complete,
        }],
    }
}

pub(crate) fn i_am_a_gps(save: &SaveGame) -> AchievementEvaluation {
    discovered_roads(save, I_AM_A_GPS_TARGET_KM)
}

pub(crate) fn pathfinder(save: &SaveGame) -> AchievementEvaluation {
    discovered_roads(save, PATHFINDER_TARGET_KM)
}

/// Uses `player.discovary_distance`, read as km of discovered road.
fn discovered_roads(save: &SaveGame, target_km: u32) -> AchievementEvaluation {
    let km = save
        .career
        .discovered_road_km
        .map(|km| km.max(0.0).floor() as u32);
    counter(km, target_km, "km", "Discovered roads")
}

pub(crate) fn choo_choo(save: &SaveGame) -> AchievementEvaluation {
    ferry_crossings(save, true)
}

pub(crate) fn sardine(save: &SaveGame) -> AchievementEvaluation {
    ferry_crossings(save, false)
}

fn ferry_crossings(save: &SaveGame, train: bool) -> AchievementEvaluation {
    let Some(crossings) = &save.career.ferry_crossings else {
        return missing_data(1, "crossings", "Ferry log");
    };
    let used = crossings
        .iter()
        .filter(|crossing| crossing.is_train() == train)
        .map(|crossing| {
            format!(
                "{} → {} ({}×)",
                crossing.from, crossing.to, crossing.use_count
            )
        })
        .collect::<Vec<_>>();
    let current = used.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, 1),
        progress: AchievementProgress {
            current,
            target: 1,
            unit: "crossings",
        },
        evidence: vec![AchievementEvidence {
            label: if train {
                "Channel Tunnel crossings".to_string()
            } else {
                "Ferry crossings".to_string()
            },
            value: summarize_list(&used),
            items: used.clone(),
            complete: current >= 1,
        }],
    }
}

pub(crate) fn head_hunter(save: &SaveGame) -> AchievementEvaluation {
    let Some(agencies) = &save.career.unlocked_recruitments else {
        return missing_data(
            HEAD_HUNTER_TARGET_AGENCIES,
            "agencies",
            "Recruitment agencies",
        );
    };
    let agencies = agencies.iter().cloned().collect::<BTreeSet<_>>();
    set_achievement(
        &agencies,
        HEAD_HUNTER_TARGET_AGENCIES,
        "agencies",
        "Discovered agencies",
    )
}

pub(crate) fn from_the_comfort_of_your_home(save: &SaveGame) -> AchievementEvaluation {
    counter(
        save.career.trucks_bought_online.map(saturating_u32),
        1,
        "trucks",
        "Trucks bought online",
    )
}

pub(crate) fn diesel_no_petrol(save: &SaveGame) -> AchievementEvaluation {
    counter(
        save.career.gas_station_visits.map(saturating_u32),
        1,
        "visits",
        "Filling station visits",
    )
}

pub(crate) fn property_magnate(save: &SaveGame) -> AchievementEvaluation {
    let Some(garages) = &save.career.garages else {
        return missing_data(PROPERTY_MAGNATE_TARGET_GARAGES, "garages", "Garages");
    };
    let owned = garages
        .iter()
        .filter(|garage| garage.size.is_owned())
        .map(|garage| garage.city.clone())
        .collect::<BTreeSet<_>>();
    set_achievement(
        &owned,
        PROPERTY_MAGNATE_TARGET_GARAGES,
        "garages",
        "Owned garages",
    )
}

pub(crate) fn go_nuclear(save: &SaveGame) -> AchievementEvaluation {
    deliveries_to_company(
        save,
        "nucleon",
        &FRENCH_NUCLEAR_PLANT_CITIES,
        3,
        "nuclear plants",
    )
}

pub(crate) fn whatever_floats_your_boat(save: &SaveGame) -> AchievementEvaluation {
    deliveries_to_company(
        save,
        "cont_port",
        &SCANDINAVIA_CONTAINER_PORT_CITIES,
        1,
        "container ports",
    )
}

pub(crate) fn captain(save: &SaveGame) -> AchievementEvaluation {
    deliveries_to_company(save, "c_navale", &[], 1, "shipyards")
}

/// Distinct destination cities of deliveries to `company` (optionally only in
/// `cities`).
fn deliveries_to_company(
    save: &SaveGame,
    company: &str,
    cities: &[&str],
    target: u32,
    unit: &'static str,
) -> AchievementEvaluation {
    let reached = save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| company_id(&entry.destination_company) == company)
        .map(|entry| company_city(&entry.destination_company).to_string())
        .filter(|city| cities.is_empty() || cities.contains(&city.as_str()))
        .collect::<BTreeSet<_>>();
    let mut evaluation = set_achievement(&reached, target, unit, "Delivered to");
    if !cities.is_empty() {
        let missing = cities
            .iter()
            .filter(|city| !reached.contains(**city))
            .map(|city| (*city).to_string())
            .collect::<BTreeSet<_>>();
        evaluation.evidence.push(AchievementEvidence {
            label: "Not yet".to_string(),
            value: if missing.is_empty() {
                "none".to_string()
            } else {
                summarize_set(&missing, SUMMARY_EVIDENCE_LIMIT)
            },
            items: Vec::new(),
            complete: missing.is_empty(),
        });
    }
    evaluation
}

/// Progress towards `target` distinct values, listing them as evidence.
fn set_achievement(
    values: &BTreeSet<String>,
    target: u32,
    unit: &'static str,
    label: &str,
) -> AchievementEvaluation {
    let current = values.len() as u32;
    AchievementEvaluation {
        status: status_from_progress(current, target),
        progress: AchievementProgress {
            current,
            target,
            unit,
        },
        evidence: vec![AchievementEvidence {
            label: label.to_string(),
            value: summarize_set(values, SUMMARY_EVIDENCE_LIMIT),
            items: set_items(values),
            complete: current >= target,
        }],
    }
}

/// Progress of a single save counter towards `target`.
fn counter(
    value: Option<u32>,
    target: u32,
    unit: &'static str,
    label: &str,
) -> AchievementEvaluation {
    let Some(current) = value else {
        return missing_data(target, unit, label);
    };
    AchievementEvaluation {
        status: status_from_progress(current, target),
        progress: AchievementProgress {
            current,
            target,
            unit,
        },
        evidence: vec![AchievementEvidence {
            label: label.to_string(),
            value: format!("{current} {unit}"),
            items: Vec::new(),
            complete: current >= target,
        }],
    }
}

fn missing_data(target: u32, unit: &'static str, label: &str) -> AchievementEvaluation {
    AchievementEvaluation {
        status: AchievementStatus::InProgress,
        progress: AchievementProgress {
            current: 0,
            target,
            unit,
        },
        evidence: vec![AchievementEvidence {
            label: label.to_string(),
            value: "not found in this save".to_string(),
            items: Vec::new(),
            complete: false,
        }],
    }
}

fn saturating_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

pub(crate) fn size_matters(save: &SaveGame) -> AchievementEvaluation {
    let deliveries = oversize_deliveries(save).count() as u32;
    let cargos = oversize_cargos(save);

    AchievementEvaluation {
        status: status_from_progress(deliveries, 1),
        progress: AchievementProgress {
            current: deliveries,
            target: 1,
            unit: "oversize deliveries",
        },
        evidence: vec![AchievementEvidence {
            label: "Oversize cargoes".to_string(),
            value: summarize_set(&cargos, SUMMARY_EVIDENCE_LIMIT),
            items: set_items(&cargos),
            complete: deliveries >= 1,
        }],
    }
}

pub(crate) fn the_bigger_the_better(save: &SaveGame) -> AchievementEvaluation {
    let cargos = oversize_cargos(save);
    let current = cargos.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, THE_BIGGER_THE_BETTER_TARGET_CARGOS),
        progress: AchievementProgress {
            current,
            target: THE_BIGGER_THE_BETTER_TARGET_CARGOS,
            unit: "oversize cargoes",
        },
        evidence: vec![AchievementEvidence {
            label: "Oversize cargoes".to_string(),
            value: summarize_set(&cargos, SUMMARY_EVIDENCE_LIMIT),
            items: set_items(&cargos),
            complete: current >= THE_BIGGER_THE_BETTER_TARGET_CARGOS,
        }],
    }
}

pub(crate) fn giant(save: &SaveGame) -> AchievementEvaluation {
    oversize_cargo_delivered(save, "silo")
}

pub(crate) fn not_a_canoe(save: &SaveGame) -> AchievementEvaluation {
    oversize_cargo_delivered(save, "boat")
}

pub(crate) fn volvo_trucks_lover(save: &SaveGame) -> AchievementEvaluation {
    cargo_delivered(save, "volvo_tr")
}

pub(crate) fn scania_trucks_lover(save: &SaveGame) -> AchievementEvaluation {
    cargo_delivered(save, "scania_tr")
}

pub(crate) fn turkish_delight(save: &SaveGame) -> AchievementEvaluation {
    let qualifying = save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| {
            company_city(&entry.source_company) == ISTANBUL
                && entry.distance_km >= TURKISH_DELIGHT_MIN_DISTANCE_KM
        })
        .map(route_summary)
        .collect::<Vec<_>>();
    let current = qualifying.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, TURKISH_DELIGHT_TARGET_DELIVERIES),
        progress: AchievementProgress {
            current,
            target: TURKISH_DELIGHT_TARGET_DELIVERIES,
            unit: "deliveries",
        },
        evidence: vec![AchievementEvidence {
            label: "From Istanbul, 2,500 km or more".to_string(),
            value: summarize_list(&qualifying),
            items: qualifying.clone(),
            complete: current >= TURKISH_DELIGHT_TARGET_DELIVERIES,
        }],
    }
}

pub(crate) fn orient_express(save: &SaveGame) -> AchievementEvaluation {
    route_legs(save, &ORIENT_EXPRESS_LEGS, true)
}

pub(crate) fn iberian_pilgrimage(save: &SaveGame) -> AchievementEvaluation {
    route_legs(save, &IBERIAN_PILGRIMAGE_LEGS, false)
}

fn oversize_deliveries(save: &SaveGame) -> impl Iterator<Item = &DeliveryLogEntry> {
    save.delivery_log
        .entries
        .iter()
        .filter(|entry| entry.job_type == OVERSIZE_JOB_TYPE)
}

fn oversize_cargos(save: &SaveGame) -> BTreeSet<String> {
    oversize_deliveries(save)
        .map(|entry| cargo_id(&entry.cargo).to_string())
        .filter(|cargo| !cargo.is_empty())
        .collect()
}

/// Special Transport cargo IDs are matched by a distinctive token
/// (`silo`, `785` for the Cat 785C chassis, `boat` for the service boat).
fn oversize_cargo_delivered(save: &SaveGame, token: &str) -> AchievementEvaluation {
    let cargos = oversize_cargos(save);
    let matching = cargos
        .iter()
        .filter(|cargo| cargo.contains(token))
        .cloned()
        .collect::<BTreeSet<_>>();
    let deliveries = oversize_deliveries(save)
        .filter(|entry| cargo_id(&entry.cargo).contains(token))
        .count() as u32;
    let complete = deliveries >= 1;

    AchievementEvaluation {
        status: status_from_progress(deliveries, 1),
        progress: AchievementProgress {
            current: deliveries,
            target: 1,
            unit: "deliveries",
        },
        evidence: vec![
            AchievementEvidence {
                label: "Matching cargo".to_string(),
                value: summarize_set(&matching, SUMMARY_EVIDENCE_LIMIT),
                items: Vec::new(),
                complete,
            },
            AchievementEvidence {
                label: "All oversize cargoes".to_string(),
                value: summarize_set(&cargos, SUMMARY_EVIDENCE_LIMIT),
                items: set_items(&cargos),
                complete,
            },
        ],
    }
}

fn cargo_delivered(save: &SaveGame, cargo: &str) -> AchievementEvaluation {
    let deliveries = save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| cargo_id(&entry.cargo) == cargo)
        .map(route_summary)
        .collect::<Vec<_>>();
    let current = deliveries.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, 1),
        progress: AchievementProgress {
            current,
            target: 1,
            unit: "deliveries",
        },
        evidence: vec![AchievementEvidence {
            label: format!("{cargo} deliveries"),
            value: summarize_list(&deliveries),
            items: deliveries.clone(),
            complete: current >= 1,
        }],
    }
}

/// Checks which `(source city, destination city)` legs appear in the delivery
/// log, in the given direction. When `in_order` is set, each leg only counts
/// if it was delivered after the previous one (the log is oldest first).
fn route_legs(
    save: &SaveGame,
    legs: &[(&'static str, &'static str)],
    in_order: bool,
) -> AchievementEvaluation {
    let entries = &save.delivery_log.entries;
    let is_leg = |entry: &DeliveryLogEntry, from: &str, to: &str| {
        company_city(&entry.source_company) == from
            && company_city(&entry.destination_company) == to
    };
    let mut next_index = 0;
    let mut chain_broken = false;
    let evidence = legs
        .iter()
        .map(|(from, to)| {
            let complete = if in_order {
                let found = (!chain_broken)
                    .then(|| {
                        entries[next_index..]
                            .iter()
                            .position(|entry| is_leg(entry, from, to))
                    })
                    .flatten();
                match found {
                    Some(offset) => {
                        next_index += offset + 1;
                        true
                    }
                    None => {
                        chain_broken = true;
                        false
                    }
                }
            } else {
                entries.iter().any(|entry| is_leg(entry, from, to))
            };
            let count = entries
                .iter()
                .filter(|entry| is_leg(entry, from, to))
                .count();
            AchievementEvidence {
                label: format!("{from} → {to}"),
                value: match (count, complete) {
                    (0, _) => "missing".to_string(),
                    (_, true) => format!("{count} deliveries"),
                    (_, false) => format!("{count} deliveries, but not after the previous leg"),
                },
                items: Vec::new(),
                complete,
            }
        })
        .collect::<Vec<_>>();
    let current = evidence.iter().filter(|item| item.complete).count() as u32;
    let target = legs.len() as u32;

    AchievementEvaluation {
        status: status_from_progress(current, target),
        progress: AchievementProgress {
            current,
            target,
            unit: "routes",
        },
        evidence,
    }
}

fn route_summary(entry: &DeliveryLogEntry) -> String {
    format!(
        "{} → {}, {}, {} km",
        company_city(&entry.source_company),
        company_city(&entry.destination_company),
        cargo_id(&entry.cargo),
        entry.distance_km
    )
}

fn summarize_list(values: &[String]) -> String {
    if values.is_empty() {
        return "missing".to_string();
    }
    let mut summary = values
        .iter()
        .take(SUMMARY_EVIDENCE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    if values.len() > SUMMARY_EVIDENCE_LIMIT {
        summary.push(format!("+{} more", values.len() - SUMMARY_EVIDENCE_LIMIT));
    }
    summary.join("; ")
}

fn status_from_progress(current: u32, target: u32) -> AchievementStatus {
    if current >= target {
        AchievementStatus::Complete
    } else {
        AchievementStatus::InProgress
    }
}

pub fn is_owned_truck_job_type(job_type: &str) -> bool {
    OWNED_TRUCK_JOB_TYPES.contains(&job_type)
}

pub fn achievement_categories_for_cargo(cargo: &str) -> BTreeSet<&'static str> {
    let Some(metadata) = CARGOS
        .iter()
        .find(|metadata| metadata.id == cargo_id(cargo))
    else {
        return BTreeSet::new();
    };
    achievement_categories_for_metadata(metadata)
}

fn achievement_categories_for_metadata(metadata: &CargoMetadata) -> BTreeSet<&'static str> {
    let mut categories = BTreeSet::new();
    if metadata.groups.contains(&"machinery") {
        categories.insert("Machinery");
    }
    if metadata.adr_class.is_some() || metadata.groups.contains(&"adr") {
        categories.insert("ADR cargo");
    }
    if metadata.groups.contains(&"containers") || metadata.trailer_categories.contains(&"container")
    {
        categories.insert("Container");
    }
    if metadata.groups.contains(&"refrigerated")
        || metadata.trailer_categories.contains(&"refrigerated")
    {
        categories.insert("Refrigerated");
    }
    if metadata.groups.contains(&"liquid") || metadata.trailer_categories.contains(&"tr_tank") {
        categories.insert("Liquid cargo");
    }
    if metadata.fragility.is_some()
        || metadata.id == "glass_packed"
        || metadata.groups.contains(&"inloader")
    {
        categories.insert("Fragile cargo");
    }
    if metadata.groups.contains(&"construction") {
        categories.insert("Construction");
    }
    if metadata.groups.contains(&"bulk") || metadata.trailer_categories.contains(&"bulk") {
        categories.insert("Bulk cargo");
    }
    categories
}

fn display_truck_brand(brand: &str) -> &str {
    match brand {
        "daf" => "DAF",
        "iveco" => "Iveco",
        "man" => "MAN",
        "mercedes" => "Mercedes-Benz",
        "renault" => "Renault",
        "scania" => "Scania",
        "volvo" => "Volvo",
        "volkswagen" => "Volkswagen",
        _ => brand,
    }
}

fn summarize_set(values: &BTreeSet<String>, limit: usize) -> String {
    if values.is_empty() {
        return "missing".to_string();
    }

    let mut summary = values.iter().take(limit).cloned().collect::<Vec<_>>();
    if values.len() > limit {
        summary.push(format!("+{} more", values.len() - limit));
    }
    summary.join(", ")
}

fn set_items(values: &BTreeSet<String>) -> Vec<String> {
    values.iter().cloned().collect()
}

fn delivery_summary(entry: &DeliveryLogEntry) -> String {
    format!(
        "{}, {} km, {}",
        cargo_id(&entry.cargo),
        entry.distance_km,
        format_revenue(entry.revenue)
    )
}

fn format_revenue(revenue: f64) -> String {
    if revenue.fract() == 0.0 {
        format!("{revenue:.0} EUR")
    } else {
        format!("{revenue:.2} EUR")
    }
}

fn is_profit_hunter_delivery(entry: &DeliveryLogEntry) -> bool {
    entry.revenue > PROFIT_HUNTER_TARGET_REVENUE
        && entry.distance_km >= PROFIT_HUNTER_TARGET_DISTANCE_KM
}

fn profit_hunter_criteria_met(entry: &DeliveryLogEntry) -> u32 {
    u32::from(entry.revenue > PROFIT_HUNTER_TARGET_REVENUE)
        + u32::from(entry.distance_km >= PROFIT_HUNTER_TARGET_DISTANCE_KM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ets2::{
        CareerStats, DeliveryLog, DeliveryLogEntry, FerryCrossing, Garage, GarageSize,
    };

    #[test]
    fn evaluates_experience_beats_all() {
        let save = save_with_entries(vec![
            entry("cargo.digger1000", 200, "vehicle.man.tgx", "quick"),
            entry("cargo.hydrogen", 200, "vehicle.daf.xf", "quick"),
            entry("cargo.apples_c", 200, "vehicle.iveco.hiway", "quick"),
            entry("cargo.canned_beef", 200, "vehicle.mercedes.actros", "quick"),
            entry("cargo.acid", 200, "vehicle.renault.t", "quick"),
            entry("cargo.glass_packed", 200, "vehicle.scania.r", "quick"),
            entry("cargo.bricks", 200, "vehicle.volvo.fh16", "quick"),
            entry("cargo.gravel", 200, "vehicle.man.tgx", "quick"),
        ]);

        let achievement = achievement_by_id(&save, "experience_beats_all");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 8);
        assert!(achievement
            .evidence
            .iter()
            .all(|evidence| evidence.complete));
        assert!(achievement
            .evidence
            .iter()
            .any(|evidence| evidence.label == "Bulk cargo"
                && evidence.items == vec!["gravel".to_string()]));
    }

    #[test]
    fn evaluates_test_drive_limited_owned_truck_jobs_only() {
        let save = save_with_entries(vec![
            entry("cargo.gravel", 999, "vehicle.man.tgx", "cargo"),
            entry("cargo.gravel", 1000, "vehicle.daf.xf", "external"),
            entry("cargo.gravel", 1001, "vehicle.mercedes.actros", "compn"),
            entry("cargo.gravel", 1002, "vehicle.renault.t", "on_compn"),
            entry("cargo.gravel", 1003, "vehicle.scania.r", "cargo"),
            entry("cargo.gravel", 5000, "vehicle.volvo.fh16", "quick"),
            entry("cargo.gravel", 5000, "vehicle.iveco.hiway", "freerm"),
            entry("cargo.gravel", 5000, "vehicle.iveco.hiway", "spec_oversize"),
            entry("cargo.gravel", 5000, "vehicle.iveco.hiway", "unknown"),
        ]);

        let achievement = achievement_by_id(&save, "test_drive_limited");

        assert_eq!(
            achievement.display_name,
            "Test Drive Limited (5 Truck Brands)"
        );
        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 5);
        assert!(achievement
            .evidence
            .iter()
            .any(|evidence| evidence.label == "Mercedes-Benz" && evidence.complete));
        assert!(!achievement
            .evidence
            .iter()
            .any(|evidence| evidence.label == "Volvo"));
        // Brands are listed by distance, longest first; 999 km is enough.
        let brands = achievement
            .evidence
            .iter()
            .map(|evidence| {
                (
                    evidence.label.as_str(),
                    evidence.value.as_str(),
                    evidence.complete,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(brands[0], ("Scania", "1003 km", true));
        assert_eq!(brands[4], ("MAN", "999 km", true));

        let short = achievement_by_id(
            &save_with_entries(vec![entry("cargo.gravel", 998, "vehicle.man.tgx", "cargo")]),
            "test_drive_limited",
        );
        assert_eq!(short.evidence[0].value, "998 km");
        assert!(!short.evidence[0].complete);
    }

    #[test]
    fn evaluates_all_is_possible() {
        let mut entries = (0..32)
            .map(|index| {
                entry(
                    &format!("cargo.cargo_{index}"),
                    200,
                    "vehicle.man.tgx",
                    "quick",
                )
            })
            .collect::<Vec<_>>();
        entries.push(entry("", 200, "vehicle.man.tgx", "quick"));
        let save = save_with_entries(entries);

        let achievement = achievement_by_id(&save, "all_is_possible");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 32);
        assert_eq!(achievement.progress.target, 30);
        assert!(achievement.evidence[0].complete);
        assert!(achievement.evidence[0].value.contains("+2 more"));
        assert_eq!(achievement.evidence[0].items.len(), 32);
        assert!(!achievement.evidence[0]
            .items
            .iter()
            .any(|item| item.starts_with('+')));
    }

    #[test]
    fn evaluates_reliable_contractor() {
        let mut entries = (0..8)
            .map(|index| DeliveryLogEntry {
                source_company: format!("company.volatile.source_{index}.amsterdam"),
                destination_company: format!("company.volatile.destination_{index}.rotterdam"),
                ..entry("cargo.gravel", 200, "vehicle.man.tgx", "quick")
            })
            .collect::<Vec<_>>();
        entries.push(DeliveryLogEntry {
            source_company: "".to_string(),
            destination_company: "company.volatile.source_0.amsterdam".to_string(),
            ..entry("cargo.gravel", 200, "vehicle.man.tgx", "quick")
        });
        let save = save_with_entries(entries);

        let achievement = achievement_by_id(&save, "reliable_contractor");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 16);
        assert_eq!(achievement.progress.target, 15);
        assert!(achievement.evidence[0].value.contains("source_0"));
        assert_eq!(achievement.evidence[0].items.len(), 16);
        assert!(achievement.evidence[0]
            .items
            .iter()
            .any(|item| item == "source_0"));
    }

    #[test]
    fn evaluates_long_hauler() {
        let save = save_with_entries(vec![
            entry("cargo.gravel", 2000, "vehicle.man.tgx", "quick"),
            entry("cargo.canned_beef", 2001, "vehicle.scania.r", "quick"),
        ]);

        let achievement = achievement_by_id(&save, "long_hauler");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 2001);
        assert_eq!(achievement.progress.target, 2000);
        assert_eq!(
            achievement.evidence[0].value,
            "canned_beef, 2001 km, 1000 EUR"
        );
    }

    #[test]
    fn evaluates_profit_hunter_as_single_qualifying_job() {
        let save = save_with_entries(vec![
            revenue_entry("cargo.gravel", 2300, 130_001.0),
            revenue_entry("cargo.canned_beef", 3000, 100_000.0),
            revenue_entry("cargo.bricks", 1000, 180_000.0),
        ]);

        let achievement = achievement_by_id(&save, "profit_hunter");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 2);
        assert_eq!(achievement.evidence[0].label, "Qualifying job");
        assert_eq!(achievement.evidence[0].value, "gravel, 2300 km, 130001 EUR");
    }

    #[test]
    fn profit_hunter_stays_in_progress_when_requirements_are_split_across_jobs() {
        let save = save_with_entries(vec![
            revenue_entry("cargo.canned_beef", 3000, 100_000.0),
            revenue_entry("cargo.bricks", 1000, 180_000.0),
        ]);

        let achievement = achievement_by_id(&save, "profit_hunter");

        assert_eq!(achievement.status, AchievementStatus::InProgress);
        assert_eq!(achievement.progress.current, 1);
        assert_eq!(achievement.evidence[0].label, "Best revenue job");
    }

    #[test]
    fn registry_lists_every_steam_achievement_grouped_by_set() {
        let registry = evaluate_achievements(&save_with_entries(Vec::new()));

        assert_eq!(registry.achievements.len(), 114);
        assert_eq!(registry.groups.len(), 16);
        assert_eq!(registry.groups[0].id, "base_game");
        assert_eq!(registry.groups[0].name, "Base Game");
        assert_eq!(registry.groups[0].steam_dlc_app_id, None);

        let count_in = |group: &str| {
            registry
                .achievements
                .iter()
                .filter(|achievement| achievement.group == group)
                .count()
        };
        assert_eq!(count_in("base_game"), 29);
        assert_eq!(count_in("special_transport"), 8);
        assert_eq!(count_in("driving_academy"), 10);
        assert_eq!(count_in("iceland"), 4);
        assert_eq!(count_in("coaches"), 4);
        let coaches = registry.groups.last().unwrap();
        assert_eq!(coaches.id, "coaches");
        assert_eq!(coaches.name, "Coaches Expansion Achievements");
        assert_eq!(coaches.steam_group_id, 15);
        assert_eq!(coaches.steam_dlc_app_id, Some(3809270));
        assert!(registry
            .achievements
            .iter()
            .filter(|achievement| achievement.group == "coaches")
            .all(|achievement| achievement.status == AchievementStatus::NotTracked));
        assert_eq!(
            registry
                .groups
                .iter()
                .map(|group| count_in(group.id))
                .sum::<usize>(),
            114
        );

        assert_eq!(registry.achievements[0].steam_api_name, "map_discover_60");
        assert_eq!(
            registry.achievements[28].steam_api_name,
            "unload_difficulty"
        );

        let ids = registry
            .achievements
            .iter()
            .map(|achievement| achievement.id)
            .collect::<BTreeSet<_>>();
        let api_names = registry
            .achievements
            .iter()
            .map(|achievement| achievement.steam_api_name)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), 114);
        assert_eq!(api_names.len(), 114);
    }

    #[test]
    fn delivery_log_achievements_are_tracked_and_others_are_marked_not_tracked() {
        let registry = evaluate_achievements(&save_with_entries(Vec::new()));
        let tracked = registry
            .achievements
            .iter()
            .filter(|achievement| achievement.status != AchievementStatus::NotTracked)
            .map(|achievement| achievement.id)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            tracked,
            BTreeSet::from([
                // Base game
                "all_is_possible",
                "choo_choo",
                "diesel_no_petrol",
                "experience_beats_all",
                "from_the_comfort_of_your_home",
                "head_hunter",
                "i_am_a_gps",
                "long_hauler",
                "pathfinder",
                "profit_hunter",
                "property_magnate",
                "reliable_contractor",
                "sardine",
                "test_drive_limited",
                // DLC sets
                "captain",
                "giant",
                "go_nuclear",
                "iberian_pilgrimage",
                "not_a_canoe",
                "orient_express",
                "scania_trucks_lover",
                "size_matters",
                "the_bigger_the_better",
                "turkish_delight",
                "volvo_trucks_lover",
                "whatever_floats_your_boat",
            ])
        );

        let untracked = achievement_by_id(&save_with_entries(Vec::new()), "national_company");
        assert_eq!(untracked.display_name, "National Company");
        assert_eq!(untracked.status, AchievementStatus::NotTracked);
        assert_eq!(untracked.evidence.len(), 1);
        assert!(!untracked.evidence[0].complete);
        assert!(untracked.evidence[0].value.contains("cities to countries"));

        // Tracked achievements whose save data is absent stay in progress.
        let no_garages = achievement_by_id(&save_with_entries(Vec::new()), "property_magnate");
        assert_eq!(no_garages.status, AchievementStatus::InProgress);
        assert_eq!(no_garages.evidence[0].value, "not found in this save");

        let academy = achievement_by_id(&save_with_entries(Vec::new()), "double_trouble");
        assert_eq!(academy.group, "driving_academy");
        assert_eq!(academy.status, AchievementStatus::NotTracked);
    }

    #[test]
    fn evaluates_special_transport_achievements_from_oversize_jobs() {
        let mut entries = vec![
            entry("cargo.silo", 400, "vehicle.volvo.fh16", "spec_oversize"),
            entry("cargo.silo", 380, "vehicle.volvo.fh16", "spec_oversize"),
            entry(
                "cargo.pilot_boat",
                300,
                "vehicle.volvo.fh16",
                "spec_oversize",
            ),
            // A regular job never counts towards Special Transport achievements.
            entry("cargo.cat_785c", 300, "vehicle.volvo.fh16", "cargo"),
        ];
        entries.extend((0..8).map(|index| {
            entry(
                &format!("cargo.oversize_{index}"),
                300,
                "vehicle.volvo.fh16",
                "spec_oversize",
            )
        }));
        let save = save_with_entries(entries);

        let size_matters = achievement_by_id(&save, "size_matters");
        assert_eq!(size_matters.status, AchievementStatus::Complete);
        assert_eq!(size_matters.progress.current, 11);

        let bigger = achievement_by_id(&save, "the_bigger_the_better");
        assert_eq!(bigger.status, AchievementStatus::Complete);
        assert_eq!(bigger.progress.current, 10);
        assert!(!bigger.evidence[0].items.contains(&"cat_785c".to_string()));

        assert_eq!(
            achievement_by_id(&save, "giant").status,
            AchievementStatus::Complete
        );
        assert_eq!(achievement_by_id(&save, "giant").progress.current, 2);
        assert_eq!(
            achievement_by_id(&save, "not_a_canoe").status,
            AchievementStatus::Complete
        );
        // The Haul Truck Chassis cargo ID is unconfirmed, so it stays untracked.
        assert_eq!(
            achievement_by_id(&save, "big_brother").status,
            AchievementStatus::NotTracked
        );
    }

    #[test]
    fn evaluates_truck_factory_cargo_achievements() {
        let save = save_with_entries(vec![DeliveryLogEntry {
            source_company: "company.volatile.volvo_fac.goteborg".to_string(),
            ..entry("cargo.volvo_tr", 500, "vehicle.man.tgx", "quick")
        }]);

        let volvo = achievement_by_id(&save, "volvo_trucks_lover");
        assert_eq!(volvo.status, AchievementStatus::Complete);
        assert_eq!(
            volvo.evidence[0].items,
            vec!["goteborg → amsterdam, volvo_tr, 500 km".to_string()]
        );
        assert_eq!(
            achievement_by_id(&save, "scania_trucks_lover").status,
            AchievementStatus::InProgress
        );
    }

    #[test]
    fn evaluates_turkish_delight() {
        let save = save_with_entries(vec![
            route_entry("istanbul", "lisboa", 2500),
            route_entry("istanbul", "porto", 3100),
            route_entry("istanbul", "sofia", 2499),
            route_entry("ankara", "lisboa", 3000),
        ]);

        let achievement = achievement_by_id(&save, "turkish_delight");

        assert_eq!(achievement.status, AchievementStatus::InProgress);
        assert_eq!(achievement.progress.current, 2);
        assert_eq!(achievement.progress.target, 3);
    }

    #[test]
    fn evaluates_career_counter_achievements() {
        let mut save = save_with_entries(Vec::new());
        save.career = CareerStats {
            discovered_road_km: Some(25_000.7),
            sleeping_count: Some(0),
            hq_city: Some("toulouse".to_string()),
            unlocked_recruitments: Some((0..41).map(|index| format!("city_{index}")).collect()),
            trucks_bought_online: Some(2),
            gas_station_visits: Some(38),
            garages: None,
            ferry_crossings: Some(vec![
                crossing("tunnel_c", "tunnel_d"),
                crossing("hull", "esbjerg"),
                crossing("rostock", "trelleborg"),
                crossing("esbjerg", "hull"),
            ]),
        };

        let status = |id| achievement_by_id(&save, id).status;
        assert_eq!(status("i_am_a_gps"), AchievementStatus::Complete);
        assert_eq!(status("pathfinder"), AchievementStatus::InProgress);
        let pathfinder = achievement_by_id(&save, "pathfinder");
        assert_eq!(pathfinder.progress.current, 25_000);
        assert_eq!(pathfinder.progress.target, 40_000);
        assert_eq!(pathfinder.evidence[0].value, "25000 km");
        assert_eq!(status("head_hunter"), AchievementStatus::Complete);
        assert_eq!(
            status("from_the_comfort_of_your_home"),
            AchievementStatus::Complete
        );
        assert_eq!(status("diesel_no_petrol"), AchievementStatus::Complete);
        assert_eq!(status("choo_choo"), AchievementStatus::Complete);
        assert_eq!(status("sardine"), AchievementStatus::Complete);
        // Present in the save, but not confirmed enough to track.
        for id in ["zzzzz", "successfully_docked"] {
            assert_eq!(status(id), AchievementStatus::NotTracked, "{id}");
        }

        save.career.ferry_crossings = Some(vec![crossing("hull", "esbjerg")]);
        assert_eq!(
            achievement_by_id(&save, "choo_choo").status,
            AchievementStatus::InProgress
        );
    }

    #[test]
    fn evaluates_property_magnate_from_owned_garages() {
        let mut save = save_with_entries(Vec::new());
        let mut garages = vec![garage("dijon", GarageSize::Tiny, 1.0)];
        garages
            .extend((0..19).map(|index| garage(&format!("small_{index}"), GarageSize::Small, 0.5)));
        garages
            .extend((0..10).map(|index| garage(&format!("large_{index}"), GarageSize::Large, 0.5)));
        garages.push(garage("grimsby", GarageSize::NotOwned, 0.0));
        save.career.garages = Some(garages);

        let magnate = achievement_by_id(&save, "property_magnate");
        assert_eq!(magnate.progress.current, 30);
        assert_eq!(magnate.status, AchievementStatus::Complete);
        assert!(!magnate.evidence[0].items.contains(&"grimsby".to_string()));

        save.career.garages.as_mut().unwrap().remove(0);
        assert_eq!(
            achievement_by_id(&save, "property_magnate").status,
            AchievementStatus::InProgress
        );
    }

    #[test]
    fn evaluates_route_legs_by_direction_and_order() {
        let orient_express = [
            ("paris", "strasbourg"),
            ("strasbourg", "munchen"),
            ("munchen", "wien"),
            ("wien", "budapest"),
            ("budapest", "bucuresti"),
            ("bucuresti", "istanbul"),
        ];
        // The second leg is driven before the first one.
        let mut entries = vec![route_entry("strasbourg", "munchen", 360)];
        entries.extend(
            orient_express
                .iter()
                .map(|(from, to)| route_entry(from, to, 500)),
        );
        let save = save_with_entries(entries[..2].to_vec());
        let orient = achievement_by_id(&save, "orient_express");
        assert_eq!(orient.progress.current, 1);
        assert_eq!(
            orient.evidence[1].value,
            "1 deliveries, but not after the previous leg"
        );

        let orient = achievement_by_id(&save_with_entries(entries), "orient_express");
        assert_eq!(orient.status, AchievementStatus::Complete);
        assert_eq!(orient.progress.current, 6);

        let pilgrimage = achievement_by_id(
            &save_with_entries(vec![
                route_entry("sevilla", "a_coruna", 900),
                route_entry("lisboa", "a_coruna", 600),
                // Wrong direction: does not count.
                route_entry("a_coruna", "pamplona", 700),
            ]),
            "iberian_pilgrimage",
        );
        // Order does not matter for this one.
        assert_eq!(pilgrimage.progress.current, 2);
        assert_eq!(pilgrimage.status, AchievementStatus::InProgress);
    }

    #[test]
    fn evaluates_company_destination_achievements() {
        let to = |company: &str, cargo: &str| DeliveryLogEntry {
            destination_company: format!("company.volatile.{company}"),
            ..entry(cargo, 300, "vehicle.man.tgx", "cargo")
        };
        let save = save_with_entries(vec![
            to("nucleon.alban", "cargo.acid"),
            to("nucleon.golfech", "cargo.acid"),
            // Not in France.
            to("nucleon.kozloduy", "cargo.acid"),
            to("c_navale.messina", "cargo.lumber"),
            to("cont_port.esbjerg", "cargo.lumber"),
        ]);

        let nuclear = achievement_by_id(&save, "go_nuclear");
        assert_eq!(nuclear.progress.current, 2);
        assert_eq!(nuclear.status, AchievementStatus::InProgress);
        assert_eq!(nuclear.evidence[1].value, "civaux, laurent, paluel");

        assert_eq!(
            achievement_by_id(&save, "captain").status,
            AchievementStatus::Complete
        );
        let boat = achievement_by_id(&save, "whatever_floats_your_boat");
        assert_eq!(boat.status, AchievementStatus::Complete);
        assert_eq!(boat.evidence[1].value, "oslo");
    }

    #[test]
    fn owned_truck_job_type_filter_matches_v1_rules() {
        for job_type in ["cargo", "external", "compn", "on_compn"] {
            assert!(is_owned_truck_job_type(job_type));
        }
        for job_type in ["quick", "freerm", "spec_oversize", "unknown", ""] {
            assert!(!is_owned_truck_job_type(job_type));
        }
    }

    #[test]
    fn derives_achievement_categories_from_cargo_metadata() {
        assert!(achievement_categories_for_cargo("cargo.digger1000").contains("Machinery"));
        assert!(achievement_categories_for_cargo("cargo.hydrogen").contains("ADR cargo"));
        assert!(achievement_categories_for_cargo("cargo.acid").contains("ADR cargo"));
        assert!(achievement_categories_for_cargo("cargo.apples_c").contains("Container"));
        assert!(achievement_categories_for_cargo("cargo.canned_beef").contains("Refrigerated"));
        assert!(achievement_categories_for_cargo("cargo.acid").contains("Liquid cargo"));
        assert!(achievement_categories_for_cargo("cargo.acid").contains("Fragile cargo"));
        assert!(achievement_categories_for_cargo("cargo.glass_packed").contains("Fragile cargo"));
        assert!(achievement_categories_for_cargo("cargo.bricks").contains("Construction"));
        assert!(achievement_categories_for_cargo("cargo.gravel").contains("Bulk cargo"));
    }

    fn save_with_entries(entries: Vec<DeliveryLogEntry>) -> SaveGame {
        SaveGame::from_delivery_log(DeliveryLog::from_entries(entries))
    }

    fn entry(cargo: &str, distance_km: u32, truck: &str, job_type: &str) -> DeliveryLogEntry {
        DeliveryLogEntry {
            source_company: "company.volatile.lkwlog.amsterdam".to_string(),
            destination_company: "company.volatile.stokes.amsterdam".to_string(),
            cargo: cargo.to_string(),
            distance_km,
            revenue: 1000.0,
            truck: truck.to_string(),
            job_type: job_type.to_string(),
            ..DeliveryLogEntry::default()
        }
    }

    fn crossing(from: &str, to: &str) -> FerryCrossing {
        FerryCrossing {
            from: from.to_string(),
            to: to.to_string(),
            use_count: 1,
        }
    }

    fn garage(city: &str, size: GarageSize, productivity: f64) -> Garage {
        Garage {
            city: city.to_string(),
            size,
            productivity,
        }
    }

    fn route_entry(from_city: &str, to_city: &str, distance_km: u32) -> DeliveryLogEntry {
        DeliveryLogEntry {
            source_company: format!("company.volatile.source.{from_city}"),
            destination_company: format!("company.volatile.destination.{to_city}"),
            ..entry("cargo.gravel", distance_km, "vehicle.man.tgx", "cargo")
        }
    }

    fn revenue_entry(cargo: &str, distance_km: u32, revenue: f64) -> DeliveryLogEntry {
        DeliveryLogEntry {
            revenue,
            ..entry(cargo, distance_km, "vehicle.man.tgx", "quick")
        }
    }

    fn achievement_by_id(save: &SaveGame, id: &str) -> Achievement {
        evaluate_achievements(save)
            .achievements
            .into_iter()
            .find(|achievement| achievement.id == id)
            .unwrap()
    }
}
