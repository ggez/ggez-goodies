

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Dimensions {
    w: u32,
    h: u32,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Frame {
    filename: String,
    frame: Rect,
    rotated: bool,
    trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    sprite_source_size: Rect,
    #[serde(rename = "sourceSize")]
    source_size: Dimensions,
    duration: u32,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Frametag {
    name: String,
    from: u32,
    to: u32,
    direction: String,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Layer {
    name: String,
    opacity: u32,
    #[serde(rename = "blendMode")]
    blend_mode: String,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Metadata {
    app: String,
    version: String,
    format: String,
    size: Dimensions,
    scale: String, // Surely this should be a number?
    #[serde(rename = "frameTags")]
    frame_tags: Vec<Frametag>,
    layers: Vec<Layer>,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct SpritesheetData {
    frames: Vec<Frame>,
    meta: Metadata,
}
