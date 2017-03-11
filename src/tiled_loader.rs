//! Functions to load maps from the Tiled map editor.

use serde_derive;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TiledMap {
    version: f32,
    orientation: String,
    renderorder: String,
    width: usize,
    height: usize,
    tilewidth: usize,
    tileheight: usize,
    nextobjectid: usize,
}


#[cfg(test)]
mod tests {
    use serde_xml_rs;
    use serde_json;
    use super::*;

    fn get_xml_map() -> &'static str {
        include_str!("testmap.tmx")
    }

    fn get_json_map() -> &'static str {
        include_str!("testmap.json")
    }

    #[test]
    fn test_map_load() {
        let mapstr1 = get_xml_map();
        let map1: TiledMap = serde_xml_rs::deserialize(mapstr1.as_bytes()).unwrap();


        let mapstr2 = get_json_map();
        let map2: TiledMap = serde_json::from_str(mapstr2).unwrap();

        assert_eq!(map1, map2);
        println!("Map is: {:?}", map1);
        assert!(false);
    }
}
