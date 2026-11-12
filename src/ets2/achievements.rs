use std::collections::{BTreeMap, BTreeSet};

use crate::ets2::generated::cargo_metadata::CARGOS;
use crate::ets2::save::{cargo_id, company_id, truck_brand, DeliveryLogEntry, SaveGame};
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementRegistry {
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Achievement {
    pub id: &'static str,
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

/// Static description of one Steam achievement and how the tracker evaluates it.
#[derive(Debug, Clone, Copy)]
pub struct AchievementDefinition {
    pub id: &'static str,
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
    /// Listed for completeness; `requires` names the save data still needed.
    NotTracked { requires: &'static str },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementEvaluation {
    pub status: AchievementStatus,
    pub progress: AchievementProgress,
    pub evidence: Vec<AchievementEvidence>,
}

const fn delivery_log(evaluate: fn(&SaveGame) -> AchievementEvaluation) -> AchievementTracking {
    AchievementTracking::DeliveryLog(evaluate)
}

const fn not_tracked(requires: &'static str) -> AchievementTracking {
    AchievementTracking::NotTracked { requires }
}

/// All 29 base-game Euro Truck Simulator 2 achievements, in Steam order.
/// Names and descriptions follow the Steam achievement page; DLC and
/// Driving Academy achievement groups are not included.
pub const BASE_GAME_ACHIEVEMENTS: [AchievementDefinition; 29] = [
    AchievementDefinition {
        id: "i_am_a_gps",
        steam_api_name: "map_discover_60",
        display_name: "I Am a GPS",
        description: "Discover at least 20,000 kilometers of roads",
        tracking: not_tracked("discovered road data"),
    },
    AchievementDefinition {
        id: "pathfinder",
        steam_api_name: "map_discover_100",
        display_name: "Pathfinder",
        description: "Discover at least 40,000 kilometers of roads",
        tracking: not_tracked("discovered road data"),
    },
    AchievementDefinition {
        id: "choo_choo",
        steam_api_name: "use_train",
        display_name: "Choo-Choo",
        description: "Use the train to cross the channel",
        tracking: not_tracked("ferry and train usage"),
    },
    AchievementDefinition {
        id: "sardine",
        steam_api_name: "use_boat",
        display_name: "Sardine",
        description: "Utilize a boat",
        tracking: not_tracked("ferry and train usage"),
    },
    AchievementDefinition {
        id: "head_hunter",
        steam_api_name: "discover_all_agencies",
        display_name: "Head Hunter",
        description: "Discover at least 40 recruitment agencies",
        tracking: not_tracked("discovered recruitment agencies"),
    },
    AchievementDefinition {
        id: "successfully_docked",
        steam_api_name: "use_all_ports",
        display_name: "Successfully Docked",
        description: "Use at least 5 ports in the game (counts sea, river and train ports)",
        tracking: not_tracked("ferry and train usage"),
    },
    AchievementDefinition {
        id: "from_the_comfort_of_your_home",
        steam_api_name: "buy_truck_online",
        display_name: "From the Comfort of Your Home",
        description: "Buy a truck online",
        tracking: not_tracked("truck purchase history"),
    },
    AchievementDefinition {
        id: "my_precious",
        steam_api_name: "metallic_multi_color_paint",
        display_name: "My Precious",
        description: "Design and apply to your truck a custom multi-color metallic paint",
        tracking: not_tracked("truck paint job data"),
    },
    AchievementDefinition {
        id: "test_drive_limited",
        steam_api_name: "999_km_with_each_brand",
        display_name: "Test Drive Limited (5 Truck Brands)",
        description:
            "Drive 999 km during jobs with each of at least 5 truck brands, only your owned trucks count",
        tracking: delivery_log(test_drive_limited),
    },
    AchievementDefinition {
        id: "just_in_time",
        steam_api_name: "job_urgent",
        display_name: "Just in Time!",
        description: "Take an urgent delivery for a minimum of 550 km and complete it with less than 30 minutes remaining",
        tracking: not_tracked("job urgency and delivery deadline data"),
    },
    AchievementDefinition {
        id: "long_hauler",
        steam_api_name: "job_2000_km",
        display_name: "Long Hauler",
        description: "Complete a delivery that was greater than 2,000 km",
        tracking: delivery_log(long_hauler),
    },
    AchievementDefinition {
        id: "reliable_contractor",
        steam_api_name: "job_15_companies",
        display_name: "Reliable Contractor",
        description: "Perform jobs for at least 15 different companies in the game",
        tracking: delivery_log(reliable_contractor),
    },
    AchievementDefinition {
        id: "profit_hunter",
        steam_api_name: "job_no_ferry",
        display_name: "Profit Hunter",
        description: "Complete a job worth over €130,000 and minimum 2,200 km",
        tracking: delivery_log(profit_hunter),
    },
    AchievementDefinition {
        id: "experience_beats_all",
        steam_api_name: "job_all_trailer_types",
        display_name: "Experience Beats All!",
        description: "Complete deliveries with following trailer types: Machinery, ADR cargo, Container, Refrigerated, Liquid cargo, Fragile cargo, Construction, Bulk cargo",
        tracking: delivery_log(experience_beats_all),
    },
    AchievementDefinition {
        id: "job_is_only_worth_it_if_its_done_well",
        steam_api_name: "perfect_delivery",
        display_name: "Job Is Only Worth It If It's Done Well!",
        description: "Complete a perfect delivery (no damage, no fines, in-time) for a job that is at least 1,000 km",
        tracking: not_tracked("per-job damage, fine and lateness data"),
    },
    AchievementDefinition {
        id: "careerist",
        steam_api_name: "5_jobs_in_a_row",
        display_name: "Careerist",
        description: "Complete 5 jobs in a row - in-time, without taking any damage to cargo and without using autoparking",
        tracking: not_tracked("per-job damage, lateness and autoparking data"),
    },
    AchievementDefinition {
        id: "friends_are_always_here_to_help_you",
        steam_api_name: "use_automatic_parking",
        display_name: "Friends Are Always Here to Help You",
        description: "Use automatic parking",
        tracking: not_tracked("autoparking usage"),
    },
    AchievementDefinition {
        id: "all_is_possible",
        steam_api_name: "jobs_30_cargo",
        display_name: "All Is Possible",
        description: "Take and complete jobs with at least 30 different cargo",
        tracking: delivery_log(all_is_possible),
    },
    AchievementDefinition {
        id: "minimaxer",
        steam_api_name: "jobs_20k_xp",
        display_name: "Minimaxer",
        description: "Gain 20,000 XP for several consecutive jobs with the total distance below 10,000 km",
        tracking: not_tracked("per-job XP data"),
    },
    AchievementDefinition {
        id: "honey_im_home",
        steam_api_name: "quick_travel_to_hq",
        display_name: "Honey, I'm Home",
        description: "Use quick travel to return to your headquarters",
        tracking: not_tracked("quick travel usage"),
    },
    AchievementDefinition {
        id: "national_company",
        steam_api_name: "garages_in_hq_country",
        display_name: "National Company",
        description: "Own at least one largest garage in your headquarter country",
        tracking: not_tracked("owned garage and headquarters data"),
    },
    AchievementDefinition {
        id: "property_magnate",
        steam_api_name: "garage_in_every_city",
        display_name: "Property Magnate",
        description: "Own at least 30 garages",
        tracking: not_tracked("owned garage data"),
    },
    AchievementDefinition {
        id: "working_with_the_elite",
        steam_api_name: "large_garage_productivity",
        display_name: "Working with the Elite",
        description: "Achieve 100% of productivity for at least 5 large garages at the same time",
        tracking: not_tracked("garage productivity data"),
    },
    AchievementDefinition {
        id: "swimming_in_success",
        steam_api_name: "daily_profit",
        display_name: "Swimming in Success",
        description: "Reach an average daily profit of €450,000",
        tracking: not_tracked("company profit history"),
    },
    AchievementDefinition {
        id: "performance_optimizer",
        steam_api_name: "productivity_all_garages",
        display_name: "Performance Optimizer",
        description: "Achieve at least 75% of the average garage productivity for 10 large garages in your company",
        tracking: not_tracked("garage productivity data"),
    },
    AchievementDefinition {
        id: "aspects_of_professionalism",
        steam_api_name: "maxed_male_female",
        display_name: "Aspects of Professionalism",
        description: "Have at least 10 female and 10 male employees of maximum level in your company",
        tracking: not_tracked("hired driver data"),
    },
    AchievementDefinition {
        id: "zzzzz",
        steam_api_name: "use_rest_stop",
        display_name: "Zzzzz",
        description: "Use a rest stop",
        tracking: not_tracked("rest stop usage"),
    },
    AchievementDefinition {
        id: "diesel_no_petrol",
        steam_api_name: "use_gas_station",
        display_name: "Diesel, No Petrol!",
        description: "Use a filling station",
        tracking: not_tracked("filling station usage"),
    },
    AchievementDefinition {
        id: "parking_challenge",
        steam_api_name: "unload_difficulty",
        display_name: "Parking Challenge",
        description: "Complete 20 deliveries choosing the trailer delivery option which requires reversing",
        tracking: not_tracked("trailer delivery (parking) option per job"),
    },
];

pub fn evaluate_achievements(save: &SaveGame) -> AchievementRegistry {
    AchievementRegistry {
        achievements: BASE_GAME_ACHIEVEMENTS
            .iter()
            .map(|definition| evaluate_definition(definition, save))
            .collect(),
    }
}

fn evaluate_definition(definition: &AchievementDefinition, save: &SaveGame) -> Achievement {
    let evaluation = match definition.tracking {
        AchievementTracking::DeliveryLog(evaluate) => evaluate(save),
        AchievementTracking::NotTracked { requires } => AchievementEvaluation {
            status: AchievementStatus::NotTracked,
            progress: AchievementProgress {
                current: 0,
                target: 0,
                unit: "",
            },
            evidence: vec![AchievementEvidence {
                label: "Not tracked yet".to_string(),
                value: format!(
                    "Needs {requires} from the save, which the tracker does not read yet"
                ),
                items: Vec::new(),
                complete: false,
            }],
        },
    };

    Achievement {
        id: definition.id,
        steam_api_name: definition.steam_api_name,
        display_name: definition.display_name,
        description: definition.description,
        status: evaluation.status,
        progress: evaluation.progress,
        evidence: evaluation.evidence,
    }
}

fn experience_beats_all(save: &SaveGame) -> AchievementEvaluation {
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

fn test_drive_limited(save: &SaveGame) -> AchievementEvaluation {
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

    let evidence = distance_by_brand
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

fn all_is_possible(save: &SaveGame) -> AchievementEvaluation {
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

fn reliable_contractor(save: &SaveGame) -> AchievementEvaluation {
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

fn long_hauler(save: &SaveGame) -> AchievementEvaluation {
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

fn profit_hunter(save: &SaveGame) -> AchievementEvaluation {
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
    use crate::ets2::{DeliveryLog, DeliveryLogEntry};

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
    fn registry_lists_all_29_base_game_achievements_in_steam_order() {
        let registry = evaluate_achievements(&save_with_entries(Vec::new()));

        assert_eq!(registry.achievements.len(), 29);
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
        assert_eq!(ids.len(), 29);
        assert_eq!(api_names.len(), 29);
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
                "all_is_possible",
                "experience_beats_all",
                "long_hauler",
                "profit_hunter",
                "reliable_contractor",
                "test_drive_limited",
            ])
        );

        let untracked = achievement_by_id(&save_with_entries(Vec::new()), "property_magnate");
        assert_eq!(untracked.display_name, "Property Magnate");
        assert_eq!(untracked.description, "Own at least 30 garages");
        assert_eq!(untracked.status, AchievementStatus::NotTracked);
        assert_eq!(untracked.evidence.len(), 1);
        assert!(!untracked.evidence[0].complete);
        assert!(untracked.evidence[0].value.contains("owned garage data"));
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
        SaveGame {
            delivery_log: DeliveryLog::from_entries(entries),
        }
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
