pub static KNOWN_MANUFACTURERS: &[&str] = &[
    // UK / British model bus manufacturers
    "CMNL",
    "Corgi",
    "EFE",
    "EFE Original Omnibus",
    "Gilbow",
    "Northcord",
    "Oxford Diecast",
    "Britbus",
    "Creative Master Northcord",
    "Exclusive First Editions",
    "Model Bus Company",
    "B-T Models",
    "Code 3",
    "Creative Master",
    "OOC",
    "Original Omnibus Company",
    "Sun Star",
    "Base Toys",
    "Corgi Omnibus",
    "Bachmann",
    "Hornby",

    // UK / European specialist manufacturers
    "Motorart",
    "Minichamps",
    "Herpa",
    "Rietze",
    "Wiking",
    "Busch",
    "Schuco",
    "Norev",
    "Solido",
    "Eligor",
    "IXO",
    "Majorette",
    "Siku",
    "Greenlight",
    "Oxford",

    // Chinese / Asian manufacturers commonly encountered
    "KMB",
    "Tiny",
    "Tiny City",
    "Tiny World",
    "Model 1",
    "Dream Model",
    "MCM",
    "GCD",
    "GeminiJets",
    "Gemini200",
    "GeminiMacs",

    // Other model manufacturers
    "Tekno",
    "NZG",
    "Conrad",
    "Cursor",
    "Lion Toys",
    "Holland Oto",
    "Universal Hobbies",
    "Norscot",
    "First Gear",
    "Motorart",
    "Replicars",
    "Cararama",
    "Hongwell",
    "Mondo Motors",
    "NewRay",
    "Kinsmart",
    "Welly",
    "RMZ City",
];

pub fn get_manufacturer_code(title: &str, manufacturer: &str) -> String {
    let after_manufacturer = title
        .split_once(manufacturer)
        .map(|(_, rest)| rest)
        .unwrap_or(title);

    after_manufacturer
        .split_whitespace()
        .find(|part| {
            part.chars().all(|c| c.is_ascii_alphanumeric())
                && part.chars().any(|c| c.is_ascii_digit())
                && part.len() >= 3
        })
        .unwrap_or_default()
        .to_string()
}