use std::collections::{BTreeMap, BTreeSet};

use crate::bsii_file::{BsiiFile, DataBlock, DataValue, Id};
use crate::ets2::generated::cargo_metadata::CARGOS;
use crate::ets2::CargoMetadata;

pub const COMPANY_PREFIX: &str = "company.volatile.";
pub const CARGO_PREFIX: &str = "cargo.";
pub const VEHICLE_PREFIX: &str = "vehicle.";

// Delivery log params are reverse engineered by the community:
// https://forum.scssoft.com/viewtopic.php?start=10&t=317674
// Entries are stored oldest first (params[0], the delivery end time, only grows).
const SOURCE_COMPANY_PARAM: usize = 1;
const DESTINATION_COMPANY_PARAM: usize = 2;
const CARGO_PARAM: usize = 3;
const XP_PARAM: usize = 4;
const REVENUE_PARAM: usize = 5;
const DISTANCE_PARAM: usize = 6;
const CARGO_DAMAGE_PARAM: usize = 7;
const TRUCK_PARAM: usize = 16;
const JOB_TYPE_PARAM: usize = 18;
// Only present in newer saves.
const SPECIAL_ROUTE_PARAM: usize = 19;
const CARGO_MASS_PARAM: usize = 22;
const MIN_PARAMS_LEN: usize = JOB_TYPE_PARAM + 1;

#[derive(Debug, Clone, PartialEq)]
pub struct SaveGame {
    pub delivery_log: DeliveryLog,
    pub career: CareerStats,
}

/// Career-wide counters and lists stored outside the delivery log.
/// Every field is optional so that a save missing a block still analyses.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CareerStats {
    /// `player.discovary_distance` (sic), in km.
    pub discovered_road_km: Option<f64>,
    /// `player.sleeping_count`: times the player slept at a rest stop.
    pub sleeping_count: Option<u64>,
    /// `player.hq_city`.
    pub hq_city: Option<String>,
    /// `economy.unlocked_recruitments`: cities whose recruitment agency is discovered.
    pub unlocked_recruitments: Option<Vec<String>>,
    /// `economy.trucks_bought_online`.
    pub trucks_bought_online: Option<u64>,
    /// `economy.gas_station_visit_count`.
    pub gas_station_visits: Option<u64>,
    /// All `garage` blocks; `None` when the save has none.
    pub garages: Option<Vec<Garage>>,
    /// All `ferry_log_entry` blocks; `None` when the save has no `ferry_log`.
    pub ferry_crossings: Option<Vec<FerryCrossing>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Garage {
    pub city: String,
    pub size: GarageSize,
    /// 0.0 to 1.0.
    pub productivity: f64,
}

/// `garage.status`, identified by the number of vehicle slots each status has.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GarageSize {
    NotOwned,
    /// Status 6: the 1-slot starter garage.
    Tiny,
    /// Status 2: 3 slots.
    Small,
    /// Status 3: 5 slots.
    Large,
    Other(u32),
}

impl GarageSize {
    pub fn from_status(status: u32) -> Self {
        match status {
            0 => GarageSize::NotOwned,
            2 => GarageSize::Small,
            3 => GarageSize::Large,
            6 => GarageSize::Tiny,
            other => GarageSize::Other(other),
        }
    }

    pub fn is_owned(self) -> bool {
        self != GarageSize::NotOwned
    }
}

/// One ferry or train connection the player has used (`ferry_log_entry`).
#[derive(Debug, Clone, PartialEq)]
pub struct FerryCrossing {
    pub from: String,
    pub to: String,
    pub use_count: u64,
}

impl FerryCrossing {
    /// The Channel Tunnel train between Calais and Folkestone.
    pub fn is_train(&self) -> bool {
        self.from.starts_with("tunnel") || self.to.starts_with("tunnel")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliveryLog {
    pub entries: Vec<DeliveryLogEntry>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeliveryLogEntry {
    pub source_company: String,
    pub destination_company: String,
    pub cargo: String,
    pub distance_km: u32,
    pub revenue: f64,
    pub truck: String,
    pub job_type: String,
    pub xp: u32,
    /// Fraction of cargo damage, 0.0 to 1.0.
    pub cargo_damage: f64,
    /// Special Transport route ID, empty for other jobs.
    pub special_route: String,
    pub cargo_mass_kg: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliveryAnalytics {
    pub delivery_count: usize,
    pub total_distance_km: u64,
    pub total_revenue: f64,
    pub unique_cargos: BTreeSet<String>,
    pub unique_companies: BTreeSet<String>,
    pub job_type_breakdown: BTreeMap<String, usize>,
    pub brand_distance_km: BTreeMap<String, u64>,
    pub cargo_category_coverage: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SaveGameError {
    MissingDeliveryLog,
    MissingDeliveryLogEntries,
}

impl std::fmt::Display for SaveGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveGameError::MissingDeliveryLog => write!(f, "missing delivery_log block"),
            SaveGameError::MissingDeliveryLogEntries => {
                write!(f, "missing delivery_log entries field")
            }
        }
    }
}

impl std::error::Error for SaveGameError {}

impl SaveGame {
    pub fn from_bsii(bsii: &BsiiFile<'_>) -> Result<Self, SaveGameError> {
        let delivery_log = DeliveryLog::from_bsii(bsii)?;
        let career = CareerStats::from_bsii(bsii);
        Ok(Self {
            delivery_log,
            career,
        })
    }

    pub fn from_delivery_log(delivery_log: DeliveryLog) -> Self {
        Self {
            delivery_log,
            career: CareerStats::default(),
        }
    }

    pub fn analytics(&self) -> DeliveryAnalytics {
        self.delivery_log.analytics()
    }
}

impl DeliveryLog {
    pub fn from_entries(entries: Vec<DeliveryLogEntry>) -> Self {
        Self { entries }
    }

    pub fn from_bsii(bsii: &BsiiFile<'_>) -> Result<Self, SaveGameError> {
        let entry_blocks = ordered_delivery_entry_blocks(bsii)?;
        let entries = entry_blocks
            .into_iter()
            .filter_map(|block| DeliveryLogEntry::from_block(bsii, block))
            .collect();
        Ok(Self { entries })
    }

    pub fn delivery_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_distance_km(&self) -> u64 {
        self.entries
            .iter()
            .map(|entry| u64::from(entry.distance_km))
            .sum()
    }

    pub fn total_revenue(&self) -> f64 {
        self.entries.iter().map(|entry| entry.revenue).sum()
    }

    pub fn unique_cargos(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .map(|entry| cargo_id(&entry.cargo).to_string())
            .collect()
    }

    pub fn unique_companies(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .flat_map(|entry| {
                [
                    company_id(&entry.source_company).to_string(),
                    company_id(&entry.destination_company).to_string(),
                ]
            })
            .collect()
    }

    pub fn job_type_breakdown(&self) -> BTreeMap<String, usize> {
        count_by(self.entries.iter().map(|entry| entry.job_type.as_str()))
    }

    pub fn brand_distance_km(&self) -> BTreeMap<String, u64> {
        let mut distance_by_brand = BTreeMap::new();
        for entry in &self.entries {
            *distance_by_brand
                .entry(truck_brand(&entry.truck).to_string())
                .or_insert(0) += u64::from(entry.distance_km);
        }
        distance_by_brand
    }

    pub fn cargo_category_coverage(&self) -> BTreeMap<String, usize> {
        let metadata_by_cargo = cargo_metadata_by_id();
        let mut coverage = BTreeMap::new();
        for entry in &self.entries {
            if let Some(metadata) = metadata_by_cargo.get(cargo_id(&entry.cargo)) {
                for category in metadata.trailer_categories {
                    *coverage.entry((*category).to_string()).or_insert(0) += 1;
                }
            }
        }
        coverage
    }

    pub fn analytics(&self) -> DeliveryAnalytics {
        DeliveryAnalytics {
            delivery_count: self.delivery_count(),
            total_distance_km: self.total_distance_km(),
            total_revenue: self.total_revenue(),
            unique_cargos: self.unique_cargos(),
            unique_companies: self.unique_companies(),
            job_type_breakdown: self.job_type_breakdown(),
            brand_distance_km: self.brand_distance_km(),
            cargo_category_coverage: self.cargo_category_coverage(),
        }
    }
}

impl DeliveryLogEntry {
    pub fn from_params(params: &[&str]) -> Option<Self> {
        if params.len() < MIN_PARAMS_LEN {
            return None;
        }
        Some(Self {
            source_company: params[SOURCE_COMPANY_PARAM].to_string(),
            destination_company: params[DESTINATION_COMPANY_PARAM].to_string(),
            cargo: params[CARGO_PARAM].to_string(),
            distance_km: params[DISTANCE_PARAM].parse().ok()?,
            revenue: params[REVENUE_PARAM].parse().ok()?,
            truck: params[TRUCK_PARAM].to_string(),
            job_type: params[JOB_TYPE_PARAM].to_string(),
            xp: params[XP_PARAM]
                .parse::<i64>()
                .map_or(0, |xp| xp.clamp(0, i64::from(u32::MAX)) as u32),
            cargo_damage: params[CARGO_DAMAGE_PARAM].parse().unwrap_or(0.0),
            special_route: params
                .get(SPECIAL_ROUTE_PARAM)
                .map_or_else(String::new, |route| route.to_string()),
            cargo_mass_kg: params
                .get(CARGO_MASS_PARAM)
                .and_then(|mass| mass.parse::<f64>().ok())
                .filter(|mass| *mass >= 0.0),
        })
    }

    fn from_block(bsii: &BsiiFile<'_>, block: &DataBlock<'_>) -> Option<Self> {
        let params = match block.field(bsii, "params")? {
            DataValue::StringArray(params) => params.as_slice(),
            _ => return None,
        };
        Self::from_params(params)
    }
}

impl CareerStats {
    pub fn from_bsii(bsii: &BsiiFile<'_>) -> Self {
        let economy = bsii.blocks_by_prototype_name("economy").next();
        let player = bsii.blocks_by_prototype_name("player").next();
        let economy_field = |name: &str| economy.and_then(|block| block.field(bsii, name));
        let player_field = |name: &str| player.and_then(|block| block.field(bsii, name));

        let garages = bsii
            .blocks_by_prototype_name("garage")
            .map(|block| Garage {
                city: last_id_part(&block.id),
                size: GarageSize::from_status(
                    block
                        .field(bsii, "status")
                        .and_then(value_as_u64)
                        .unwrap_or(0) as u32,
                ),
                productivity: block
                    .field(bsii, "productivity")
                    .and_then(value_as_f64)
                    .unwrap_or(0.0),
            })
            .collect::<Vec<_>>();
        let has_ferry_log = bsii.blocks_by_prototype_name("ferry_log").next().is_some();
        let ferry_crossings = bsii
            .blocks_by_prototype_name("ferry_log_entry")
            .filter_map(|block| {
                Some(FerryCrossing {
                    from: block.field(bsii, "ferry").and_then(value_as_string)?,
                    to: block.field(bsii, "connection").and_then(value_as_string)?,
                    use_count: block
                        .field(bsii, "use_count")
                        .and_then(value_as_u64)
                        .unwrap_or(0),
                })
            })
            .collect::<Vec<_>>();

        Self {
            discovered_road_km: player_field("discovary_distance").and_then(value_as_f64),
            sleeping_count: player_field("sleeping_count").and_then(value_as_u64),
            hq_city: player_field("hq_city").and_then(value_as_string),
            unlocked_recruitments: economy_field("unlocked_recruitments")
                .and_then(value_as_string_array),
            trucks_bought_online: economy_field("trucks_bought_online").and_then(value_as_u64),
            gas_station_visits: economy_field("gas_station_visit_count").and_then(value_as_u64),
            garages: (!garages.is_empty()).then_some(garages),
            ferry_crossings: (has_ferry_log || !ferry_crossings.is_empty())
                .then_some(ferry_crossings),
        }
    }
}

fn last_id_part(id: &Id) -> String {
    match id {
        Id::Named(parts) => parts.last().cloned().unwrap_or_default(),
        Id::Nameless(value) => value.to_string(),
    }
}

fn value_as_u64(value: &DataValue<'_>) -> Option<u64> {
    match value {
        DataValue::Int32(value) => u64::try_from(*value).ok(),
        DataValue::UInt32(value) | DataValue::Enum(value) => Some(u64::from(*value)),
        DataValue::UInt16(value) => Some(u64::from(*value)),
        DataValue::Int64(value) => u64::try_from(*value).ok(),
        DataValue::UInt64(value) => Some(*value),
        DataValue::Float(value) if *value >= 0.0 => Some(*value as u64),
        _ => None,
    }
}

fn value_as_f64(value: &DataValue<'_>) -> Option<f64> {
    match value {
        DataValue::Float(value) => Some(f64::from(*value)),
        DataValue::Int32(value) => Some(f64::from(*value)),
        DataValue::UInt32(value) => Some(f64::from(*value)),
        DataValue::UInt16(value) => Some(f64::from(*value)),
        DataValue::Int64(value) => Some(*value as f64),
        DataValue::UInt64(value) => Some(*value as f64),
        _ => None,
    }
}

/// Tokens decode as strings; some fields are links (IDs) that render the same
/// way in text saves, so accept those too.
fn value_as_string(value: &DataValue<'_>) -> Option<String> {
    match value {
        DataValue::String(value) => Some((*value).to_string()),
        DataValue::EncodedString(value) => Some(value.clone()),
        DataValue::Id(Id::Named(parts)) => Some(parts.join(".")),
        _ => None,
    }
}

fn value_as_string_array(value: &DataValue<'_>) -> Option<Vec<String>> {
    match value {
        DataValue::StringArray(values) => {
            Some(values.iter().map(|value| (*value).to_string()).collect())
        }
        DataValue::EncodedStringArray(values) => Some(values.clone()),
        DataValue::IdArray(ids) => Some(
            ids.iter()
                .filter_map(|id| match id {
                    Id::Named(parts) => Some(parts.join(".")),
                    Id::Nameless(_) => None,
                })
                .collect(),
        ),
        _ => None,
    }
}

fn ordered_delivery_entry_blocks<'a>(
    bsii: &'a BsiiFile<'a>,
) -> Result<Vec<&'a DataBlock<'a>>, SaveGameError> {
    let log_block = bsii
        .blocks_by_prototype_name("delivery_log")
        .next()
        .ok_or(SaveGameError::MissingDeliveryLog)?;
    let entries = match log_block.field(bsii, "entries") {
        Some(DataValue::IdArray(entries)) => entries,
        _ => return Err(SaveGameError::MissingDeliveryLogEntries),
    };

    let entry_blocks = entries
        .iter()
        .filter_map(|entry_id| {
            bsii.blocks_by_prototype_name("delivery_log_entry")
                .find(|block| &block.id == entry_id)
        })
        .collect();
    Ok(entry_blocks)
}

fn cargo_metadata_by_id() -> BTreeMap<&'static str, &'static CargoMetadata> {
    CARGOS.iter().map(|cargo| (cargo.id, cargo)).collect()
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

pub fn cargo_id(cargo: &str) -> &str {
    cargo.strip_prefix(CARGO_PREFIX).unwrap_or(cargo)
}

pub fn company_id(company: &str) -> &str {
    company
        .strip_prefix(COMPANY_PREFIX)
        .unwrap_or(company)
        .split('.')
        .next()
        .unwrap_or(company)
}

/// City ID of a `company.volatile.<company>.<city>` delivery log company.
pub fn company_city(company: &str) -> &str {
    company
        .strip_prefix(COMPANY_PREFIX)
        .and_then(|rest| rest.split_once('.'))
        .map_or("", |(_, city)| city)
}

pub fn truck_brand(truck: &str) -> &str {
    truck
        .strip_prefix(VEHICLE_PREFIX)
        .unwrap_or(truck)
        .split('.')
        .next()
        .unwrap_or(truck)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::bsii_file::{DataBlock, Id, Prototype, ValuePrototype};
    use crate::file_type::decode_until_bsii;

    #[test]
    fn extracts_delivery_log_entries_from_bsii() {
        let first_entry_id = Id::Nameless(10);
        let second_entry_id = Id::Nameless(11);
        let bsii = synthetic_bsii(vec![first_entry_id.clone(), second_entry_id.clone()]);

        let save = SaveGame::from_bsii(&bsii).unwrap();

        assert_eq!(save.delivery_log.entries.len(), 2);
        assert_eq!(save.delivery_log.entries[0].cargo, "cargo.gravel");
        assert_eq!(save.delivery_log.entries[0].distance_km, 362);
        assert_eq!(save.delivery_log.entries[0].revenue, 16930.0);
        assert_eq!(save.delivery_log.entries[1].truck, "vehicle.scania.r");
    }

    #[test]
    fn computes_delivery_analytics() {
        let log = DeliveryLog::from_entries(vec![
            entry(
                "company.volatile.lkwlog.amsterdam",
                "company.volatile.stokes.amsterdam",
                "cargo.gravel",
                362,
                16930.0,
                "vehicle.mercedes.actros",
                "quick",
            ),
            entry(
                "company.volatile.lkwlog.amsterdam",
                "company.volatile.transinet.rotterdam",
                "cargo.canned_beef",
                1482,
                4830.5,
                "vehicle.scania.r",
                "cargo",
            ),
        ]);

        let analytics = log.analytics();

        assert_eq!(analytics.delivery_count, 2);
        assert_eq!(analytics.total_distance_km, 1844);
        assert_eq!(analytics.total_revenue, 21760.5);
        assert_eq!(analytics.unique_cargos, btreeset(["canned_beef", "gravel"]));
        assert_eq!(
            analytics.unique_companies,
            btreeset(["lkwlog", "stokes", "transinet"])
        );
        assert_eq!(
            analytics.job_type_breakdown,
            btreemap([("cargo", 1), ("quick", 1)])
        );
        assert_eq!(
            analytics.brand_distance_km,
            btreemap([("mercedes", 362_u64), ("scania", 1482)])
        );
        assert_eq!(
            analytics.cargo_category_coverage,
            btreemap([("bulk", 1), ("refrigerated", 1)])
        );
    }

    #[test]
    fn parses_delivery_params_and_ids() {
        let params = [
            "605",
            "company.volatile.lkwlog.amsterdam",
            "company.volatile.stokes.amsterdam",
            "cargo.gravel",
            "16",
            "16930.000",
            "362",
            "0.000",
            "295",
            "0",
            "0",
            "1",
            "1",
            "16930",
            "0",
            "600",
            "vehicle.mercedes.actros",
            "362",
            "quick",
            "",
            "",
            "0",
            "25000.000",
        ];
        let entry = DeliveryLogEntry::from_params(&params).unwrap();

        assert_eq!(entry.source_company, "company.volatile.lkwlog.amsterdam");
        assert_eq!(company_id(&entry.source_company), "lkwlog");
        assert_eq!(company_city(&entry.source_company), "amsterdam");
        assert_eq!(company_city(""), "");
        assert_eq!(cargo_id(&entry.cargo), "gravel");
        assert_eq!(entry.distance_km, 362);
        assert_eq!(entry.revenue, 16930.0);
        assert_eq!(truck_brand(&entry.truck), "mercedes");
        assert_eq!(entry.job_type, "quick");
    }

    #[test]
    fn parses_newer_delivery_params() {
        // Shape of a 24-param entry from a 1.5x save.
        let params = [
            "8423653",
            "company.volatile.app.coimbra",
            "company.volatile.log_atlan.sevilla",
            "cargo.press_sl_val",
            "791",
            "37631",
            "671",
            "0.012",
            "997",
            "0",
            "1",
            "1",
            "0",
            "39603",
            "800",
            "8423164",
            "vehicle.scania.s_2016",
            "668",
            "spec_oversize",
            "lux_lieg1",
            "",
            "1",
            "30639.299",
            "53",
        ];

        let entry = DeliveryLogEntry::from_params(&params).unwrap();

        assert_eq!(entry.xp, 791);
        assert_eq!(entry.distance_km, 671);
        assert_eq!(entry.cargo_damage, 0.012);
        assert_eq!(entry.special_route, "lux_lieg1");
        assert_eq!(entry.cargo_mass_kg, Some(30639.299));
        assert_eq!(company_city(&entry.destination_company), "sevilla");
    }

    #[test]
    fn parses_career_stats_from_economy_player_garage_and_ferry_blocks() {
        let bsii = BsiiFile {
            header: b"BSII",
            version: 3,
            prototypes: HashMap::from([
                prototype(
                    1,
                    "economy",
                    &[
                        "unlocked_recruitments",
                        "trucks_bought_online",
                        "gas_station_visit_count",
                    ],
                ),
                prototype(
                    2,
                    "player",
                    &["hq_city", "sleeping_count", "discovary_distance"],
                ),
                prototype(3, "garage", &["status", "productivity"]),
                prototype(4, "ferry_log", &["entries"]),
                prototype(5, "ferry_log_entry", &["ferry", "connection", "use_count"]),
            ]),
            data_blocks: vec![
                DataBlock {
                    prototype_id: 1,
                    id: Id::Nameless(1),
                    data: vec![
                        DataValue::EncodedStringArray(vec![
                            "geneve".to_string(),
                            "lyon".to_string(),
                        ]),
                        DataValue::UInt32(44),
                        DataValue::UInt32(38),
                    ],
                },
                DataBlock {
                    prototype_id: 2,
                    id: Id::Nameless(2),
                    data: vec![
                        DataValue::EncodedString("toulouse".to_string()),
                        DataValue::UInt32(1),
                        DataValue::Float(44707.58),
                    ],
                },
                garage_block("dijon", 6, 1.0),
                garage_block("helsinki", 3, 0.5),
                garage_block("grimsby", 0, 0.0),
                DataBlock {
                    prototype_id: 4,
                    id: Id::Nameless(4),
                    data: vec![DataValue::IdArray(vec![Id::Nameless(5)])],
                },
                DataBlock {
                    prototype_id: 5,
                    id: Id::Nameless(5),
                    data: vec![
                        DataValue::EncodedString("tunnel_c".to_string()),
                        DataValue::EncodedString("tunnel_d".to_string()),
                        DataValue::UInt32(2),
                    ],
                },
            ],
        };

        let career = CareerStats::from_bsii(&bsii);

        assert_eq!(career.hq_city.as_deref(), Some("toulouse"));
        assert_eq!(career.sleeping_count, Some(1));
        assert!((career.discovered_road_km.unwrap() - 44707.58).abs() < 0.01);
        assert_eq!(
            career.unlocked_recruitments,
            Some(vec!["geneve".to_string(), "lyon".to_string()])
        );
        assert_eq!(career.trucks_bought_online, Some(44));
        assert_eq!(career.gas_station_visits, Some(38));
        let garages = career.garages.unwrap();
        assert_eq!(garages.len(), 3);
        assert_eq!(garages[0].city, "dijon");
        assert_eq!(garages[0].size, GarageSize::Tiny);
        assert_eq!(garages[1].size, GarageSize::Large);
        assert!(!garages[2].size.is_owned());
        let crossings = career.ferry_crossings.unwrap();
        assert_eq!(crossings.len(), 1);
        assert!(crossings[0].is_train());
        assert_eq!(crossings[0].use_count, 2);
    }

    #[test]
    fn career_stats_are_empty_when_blocks_are_missing() {
        let bsii = synthetic_bsii(vec![Id::Nameless(10)]);
        assert_eq!(CareerStats::from_bsii(&bsii), CareerStats::default());
    }

    fn prototype(
        id: u32,
        name: &'static str,
        fields: &[&'static str],
    ) -> (u32, Prototype<'static>) {
        (
            id,
            Prototype {
                id,
                name,
                value_prototypes: fields
                    .iter()
                    .map(|field| ValuePrototype {
                        type_id: 0,
                        name: field,
                        enum_values: None,
                    })
                    .collect(),
            },
        )
    }

    fn garage_block(city: &str, status: u32, productivity: f32) -> DataBlock<'static> {
        DataBlock {
            prototype_id: 3,
            id: Id::Named(vec!["garage".to_string(), city.to_string()]),
            data: vec![DataValue::UInt32(status), DataValue::Float(productivity)],
        }
    }

    #[test]
    #[ignore = "requires ignored local save at saves/my_save.sii"]
    fn extracts_ignored_local_save() {
        let path = Path::new("saves/my_save.sii");
        if !path.exists() {
            return;
        }

        let content = fs::read(path).unwrap();
        let bsii_content = decode_until_bsii(&content).unwrap();
        let bsii = BsiiFile::parse(bsii_content.as_ref()).unwrap();
        let save = SaveGame::from_bsii(&bsii).unwrap();

        assert!(save.delivery_log.delivery_count() > 0);
    }

    fn synthetic_bsii(entry_ids: Vec<Id>) -> BsiiFile<'static> {
        let entries = entry_ids
            .iter()
            .enumerate()
            .map(|(index, id)| DataBlock {
                prototype_id: 2,
                id: id.clone(),
                data: vec![DataValue::StringArray(match index {
                    0 => params(
                        "cargo.gravel",
                        "vehicle.mercedes.actros",
                        "quick",
                        "362",
                        "16930.000",
                    ),
                    _ => params(
                        "cargo.beef_meat",
                        "vehicle.scania.r",
                        "cargo",
                        "1482",
                        "4830.500",
                    ),
                })],
            })
            .collect::<Vec<_>>();

        let mut data_blocks = vec![DataBlock {
            prototype_id: 1,
            id: Id::Nameless(1),
            data: vec![DataValue::IdArray(entry_ids)],
        }];
        data_blocks.extend(entries);

        BsiiFile {
            header: b"BSII",
            version: 2,
            prototypes: HashMap::from([
                (
                    1,
                    Prototype {
                        id: 1,
                        name: "delivery_log",
                        value_prototypes: vec![ValuePrototype {
                            type_id: 0x3a,
                            name: "entries",
                            enum_values: None,
                        }],
                    },
                ),
                (
                    2,
                    Prototype {
                        id: 2,
                        name: "delivery_log_entry",
                        value_prototypes: vec![ValuePrototype {
                            type_id: 0x02,
                            name: "params",
                            enum_values: None,
                        }],
                    },
                ),
            ]),
            data_blocks,
        }
    }

    fn params(
        cargo: &'static str,
        truck: &'static str,
        job_type: &'static str,
        distance: &'static str,
        revenue: &'static str,
    ) -> Vec<&'static str> {
        vec![
            "605",
            "company.volatile.lkwlog.amsterdam",
            "company.volatile.stokes.amsterdam",
            cargo,
            "16",
            revenue,
            distance,
            "0.000",
            "295",
            "0",
            "0",
            "1",
            "1",
            revenue,
            "0",
            "600",
            truck,
            distance,
            job_type,
            "",
            "",
            "0",
            "25000.000",
        ]
    }

    fn entry(
        source_company: &str,
        destination_company: &str,
        cargo: &str,
        distance_km: u32,
        revenue: f64,
        truck: &str,
        job_type: &str,
    ) -> DeliveryLogEntry {
        DeliveryLogEntry {
            source_company: source_company.to_string(),
            destination_company: destination_company.to_string(),
            cargo: cargo.to_string(),
            distance_km,
            revenue,
            truck: truck.to_string(),
            job_type: job_type.to_string(),
            ..DeliveryLogEntry::default()
        }
    }

    fn btreeset<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
        values.into_iter().map(ToString::to_string).collect()
    }

    fn btreemap<K, V, const N: usize>(values: [(K, V); N]) -> BTreeMap<String, V>
    where
        K: ToString,
    {
        values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }
}
