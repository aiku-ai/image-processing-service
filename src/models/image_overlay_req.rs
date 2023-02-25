use serde::Deserialize;
use serde::de;

#[derive(Deserialize, Debug)]
pub struct ImageOverlayReq {
    #[serde(rename = "aikuText")]
    pub aiku_text: AikuText,

    #[serde(rename = "imageUrl")]
    pub image_url: String
}

#[derive(Deserialize, Debug)]
pub struct AikuText {
    #[serde(rename = "lineOne")]
    pub line_one: String,

    #[serde(rename = "lineTwo")]
    pub line_two: String,

    #[serde(rename = "lineThree")]
    pub line_three: String
}
