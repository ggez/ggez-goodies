

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
    spriteSourceSize: Rect,
    sourceSize: Dimensions,
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
    blendMode: String,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Metadata {
    app: String,
    version: String,
    format: String,
    size: Dimensions,
    scale: String, // Surely this should be a number?
    frameTags: Vec<Frametag>,
    layers: Vec<Layer>,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct SpritesheetData {
    frames: Vec<Frame>,
    meta: Metadata,
}
