use std::io::Cursor;
use exif::{Context, Reader, Tag};
use image::metadata::Orientation;
use serde_json::{Map, Number, Value};

pub fn extract_exif(data: &[u8], ignore_sensitive: bool) -> Option<Value> {
    let mut cursor = Cursor::new(data);

    let exif = Reader::new()
        .read_from_container(&mut cursor)
        .ok()?;

    let mut metadata = Map::new();

    for field in exif.fields() {
        let context = field.tag.context();

        // Remove location and identifying information when requested.
        if ignore_sensitive {
            let sensitive = context == Context::Gps
                || matches!(
                    field.tag,
                    Tag::BodySerialNumber
                        | Tag::CameraOwnerName
                        | Tag::LensSerialNumber
                        | Tag::ImageUniqueID
                        | Tag::MakerNote
                );

            if sensitive {
                continue;
            }
        }

        let section = match context {
            Context::Tiff => "tiff",
            Context::Exif => "exif",
            Context::Gps => "gps",
            Context::Interop => "interop",
            _ => "other",
        };

        let section_metadata = metadata
            .entry(section.to_string())
            .or_insert_with(|| Value::Object(Map::new()));

        let Some(section_metadata) = section_metadata.as_object_mut() else {
            continue;
        };

        // Use the tag's standard name rather than its verbose description.
        let key = field.tag.to_string();

        // Orientation is an unsigned integer (normally 1–8).
        // Preserve it as a number instead of a formatted string.
        let value = if field.tag == Tag::Orientation {
            field
                .value
                .get_uint(0)
                .map(|value| Value::Number(Number::from(value)))
                .unwrap_or_else(|| {
                    Value::String(
                        field.display_value().with_unit(&exif).to_string(),
                    )
                })
        } else {
            Value::String(
                field.display_value().with_unit(&exif).to_string(),
            )
        };

        section_metadata.insert(key, value);
    }

    Some(Value::Object(metadata))
}

pub fn get_photo_orientation(exif: Option<&sqlx::types::Json<Value>>) -> Orientation {
    exif
        .and_then(|exif| exif.0.get("tiff"))
        .and_then(|tiff| tiff.get("Orientation"))
        .and_then(Value::as_u64)
        .map(|value| match value {
            1 => Orientation::NoTransforms,
            2 => Orientation::FlipHorizontal,
            3 => Orientation::Rotate180,
            4 => Orientation::FlipVertical,
            5 => Orientation::Rotate90FlipH,
            6 => Orientation::Rotate90,
            7 => Orientation::Rotate270FlipH,
            8 => Orientation::Rotate270,
            _ => Orientation::NoTransforms,
        })
        .unwrap_or(Orientation::NoTransforms)
}