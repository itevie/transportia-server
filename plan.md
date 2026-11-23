Make a admin only routes so that only admins can delete things

# Tram
CREATE TABLE IF NOT EXISTS tram (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operator INTEGER REFERENCES tram_operators(id),
  fleet TEXT NOT NULL,
  livery INTEGER REFERENCES tram_livery(id),
  tram_type INTEGER REFERENCES tram_type(id),
  name TEXT,
  branding TEXT
);

CREATE TABLE IF NOT EXISTS tram_livery (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  css TEXT,
  text_colour TEXT
);

CREATE TABLE IF NOT EXISTS tram_type (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tram_operators (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  location TEXT,
  website TEXT
);

# Model Buses
CREATE TABLE IF NOT EXISTS model_buses (
id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT,
    manufacturer TEXT NOT NULL,
    scale INTEGER,
    operator TEXT,
    operator_link TEXT REFERENCES tx_operators(id),
    livery TEXT,
    livery_link INTEGER REFERENCES bt_liveries(id),
    fleet_number TEXT,
    registration TEXT,
    vehicle_link INTEGER REFERENCES bt_vehicles(id),
    route_number TEXT,
    destination TEXT,
    route_link TEXT REFERENCES tx_lines(tx_id),
    vehicle_type TEXT,
    vehicle_type_link INTEGER REFERENCES bt_vehicle_types(id),
    has_certificate INTEGER NOT NULL DEFAULT 0,
    certificate_max INTEGER NOT NULL DEFAULT 0,
    released TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    UNIQUE(product_code, manufacturer)
);

CREATE TABLE IF NOT EXISTS bus_photo_details (
id INTEGER PRIMARY KEY AUTOINCREMENT,
matched_vehicle_id INTEGER REFERENCES bt_vehicles(id),
override_match INTEGER DEFAULT 0 NOT NULL, -- Whether or not to prefer this details over matched ones
reg TEXT,
fleet TEXT,
livery_id INTEGER REFERENCES bt_liveries(id),
vehicle_type_id INTEGER REFERENCES bt_vehicle_types(id),
route_id TEXT REFERENCES tx_lines(tx_id)
);
"#;

CREATE TABLE IF NOT EXISTS model_bus_photo_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matched_model_id INTEGER REFERENCES bt_vehicles(id),
    ownership INTEGER REFERENCES model_bus_ownerships(id),
    override_match INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS model_bus_ownerships (
  id INTEGER AUTOINCREMENT PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) NOT NULL,
  model_bus INTEGER REFERENCES model_buses(id) NOT NULL,
  certificate_number INTEGER,
  bought_at TEXT,
  defects TEXT,
  created_at TEXT NOT NULL,

  UNIQUE(model_bus, certificate_number)
);

CREATE TABLE IF NOT EXISTS model_bus_photo_details (
  id INTEGER AUTOINCREMENT PRIMARY KEY,
  model_bus_link INTEGER REFERENCES model_buses(id),
  model_bus_ownership INTEGER REFERENCES model_bus_ownerships(id),
  created_at TEXT NOT NULL
);

# Photo Collections
CREATE TABLE IF NOT EXISTS photo_collections (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  public INTEGER DEFAULT 1 NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS photo_collection_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  photo_id INTEGER PRIMARY KEY REFERENCES photos(id) NOT NULL,
  added_at TEXT NOT NULL
);

# User Updates
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT NOT NULL UNIQUE,
  username TEXT NOT NULL UNIQUE,
  display_name TEXT,
  bio TEXT,
  password_hash TEXT NOT NULL,
  trusted_user INTEGER NOT NULL DEFAULT 0,
  trusted_users_can_edit_photos INTEGER NOT NULL DEFAULT 1,
)

# Vehicle Updates
CREATE TABLE IF NOT EXISTS bt_vehicles (
  id INTEGER PRIMARY KEY,
  slug TEXT NOT NULL,
  fleet TEXT NOT NULL,
  reg TEXT NOT NULL,
  previous_reg TEXT NOT NULL,
  vehicle_type_id INTEGER,
  livery_id INTEGER,
  branding TEXT NOT NULL,
  operator_id TEXT NOT NULL,
  garage_id TEXT,
  name TEXT NOT NULL,
  notes TEXT NOT NULL,
  withdrawn INTEGER NOT NULL,
  training_vehicle INTEGER NOT NULL DEFAULT 0,
  demonstrator INTEGER NOT NULL DEFAULT 0, -- <- this
  comment_thread INTEGER REFERENCES comment_threads(id)
);


# Make it so other types of checklists work

CREATE TABLE IF NOT EXISTS checklists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    checklist_type INTEGER NOT NULL, 0 = Service, 1 = Vehicle
    auto_check_photos INTEGER DEFAULT 0, -- On fetch, fetch photos and check them if found
    created_at TEXT NOT NULL
)

CREATE TABLE IF NOT EXISTS checklist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    checklist_id INTEGER REFERENCES route_checklists(id) NOT NULL,
    obj_id TEXT NOT NULL,
    added_at TEXT NOT NULL,
    completed_at TEXT,
    UNIQUE(checklist_id, obj_id)
)

# Tram details

CREATE TABLE IF NOT EXISTS tram_photo_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matched_vehicle_id INTEGER REFERENCES trams(id),
    override_match INTEGER DEFAULT 0 NOT NULL, -- Whether or not to prefer this details over matched ones
    fleet TEXT,
    livery_id INTEGER REFERENCES tram_liveries(id),
    vehicle_type_id INTEGER REFERENCES tram_types(id),
    operator_id INTEGER REFERENCES tram_operators(id),
);

// Will add location later
CREATE TABLE IF NOT EXISTS tram_stops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    operator_id INTEGER REFERENCES tram_operators(id),
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tram_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    start_stop_id INTEGER REFERENCES tram_stops(id) NOT NULL,
    end_stop_id INTEGER REFERENCES tram_stops(id) NOT NULL,
    operator_id INTEGER REFERENCES tram_operators(id) NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tram_announcement_audios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_name TEXT NOT NULL,
    audio_group TEXT NOT NULL, // "M5000", "T68", "Station", etc.
    label TEXT,
    operator_id INTEGER REFERENCES tram_operators(id),
    route_id INTEGER REFERENCES tram_routes(id),
    next_stop_id INTEGER REFERENCES tram_stops(id),
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS forums(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id) NOT NULL,
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    last_post TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS forum_posts(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    forum_id INTEGER REFERENCES forums(id) NOT NULL,
    comment_thread_id BIGINT REFERENCES comment_threads(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id) NOT NULL,
    created_at TEXT NOT NULL
);