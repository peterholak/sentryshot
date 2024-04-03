// SPDX-License-Identifier: GPL-2.0-or-later

use crate::detector::{DetectorName, Thresholds};
use common::PolygonNormalized;
use recording::{denormalize, DurationSec, FeedRateSec};
use serde::Deserialize;
use std::{num::NonZeroU16, ops::Deref};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TfliteConfig {
    //timestampOffset: time.Duration,
    pub thresholds: Thresholds,
    pub crop: Crop,
    pub mask: Mask,
    pub detector_name: DetectorName,
    pub feed_rate: FeedRateSec,
    pub duration: DurationSec,
    pub use_sub_stream: bool,
}

#[derive(Deserialize)]
struct RawConfigV1 {
    enable: bool,
    thresholds: Thresholds,
    crop: Crop,
    mask: Mask,

    #[serde(rename = "detectorName")]
    detector_name: DetectorName,

    #[serde(rename = "feedRate")]
    feed_rate: FeedRateSec,
    duration: DurationSec,

    #[serde(rename = "useSubStream")]
    use_sub_stream: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Mask {
    pub enable: bool,
    pub area: PolygonNormalized,
}

impl TfliteConfig {
    pub(crate) fn parse(raw: serde_json::Value) -> Result<Option<TfliteConfig>, serde_json::Error> {
        #[derive(Deserialize)]
        struct Temp {
            tflite: serde_json::Value,
        }
        let Ok(temp) = serde_json::from_value::<Temp>(raw) else {
            return Ok(None);
        };
        if temp.tflite == serde_json::Value::Object(serde_json::Map::new()) {
            return Ok(None);
        }

        let c: RawConfigV1 = serde_json::from_value(temp.tflite)?;

        let enable = c.enable;
        if !enable {
            return Ok(None);
        }

        //timestampOffset, err := ffmpeg.ParseTimestampOffset(c.Get("timestampOffset"))

        Ok(Some(TfliteConfig {
            //timestampOffset: timestampOffset,
            thresholds: c.thresholds,
            crop: c.crop,
            mask: c.mask,
            detector_name: c.detector_name,
            feed_rate: c.feed_rate,
            duration: c.duration,
            use_sub_stream: c.use_sub_stream,
        }))
    }
}

#[derive(Debug, Error)]
#[error("value is greater than 100")]
pub(crate) struct ParsePercentError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Percent(u8);

impl Percent {
    pub(crate) fn as_f32(self) -> f32 {
        f32::from(self.0)
    }
}

impl TryFrom<u8> for Percent {
    type Error = ParsePercentError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 100 {
            Err(ParsePercentError)
        } else {
            Ok(Self(value))
        }
    }
}

impl<'de> Deserialize<'de> for Percent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u8::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl Deref for Percent {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Crop {
    pub x: CropValue,
    pub y: CropValue,
    pub size: CropSize,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CropValue(u16);

impl CropValue {
    #[cfg(test)]
    pub(crate) fn new_testing(value: u16) -> Self {
        Self(value)
    }

    pub(crate) fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseCropValueError {
    #[error("crop size cannot be larger than 99")]
    TooLarge(u16),
}

impl TryFrom<u32> for CropValue {
    type Error = ParseCropValueError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use ParseCropValueError::*;
        let value = denormalize(value, 100);
        if value > 99 {
            return Err(TooLarge(value));
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for CropValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u32::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CropSize(NonZeroU16);

impl CropSize {
    #[cfg(test)]
    pub(crate) fn new_testing(value: NonZeroU16) -> Self {
        Self(value)
    }

    pub(crate) fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseCropSizeError {
    #[error("crop size cannot be larger than 99")]
    TooLarge(u16),

    #[error("crop size cannot be zero")]
    Zero,
}

impl TryFrom<u32> for CropSize {
    type Error = ParseCropSizeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use ParseCropSizeError::*;
        let value = denormalize(value, 100);
        if value > 99 {
            return Err(TooLarge(value));
        }
        Ok(Self(NonZeroU16::new(value).ok_or(Zero)?))
    }
}

impl<'de> Deserialize<'de> for CropSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u32::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use common::{time::Duration, PointNormalized};
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_parse_config_ok() {
        let raw = json!({
            "tflite": {
                "enable":       true,
                "thresholds":   {"5": 6},
                "crop":         [7, 8, 9],
                "mask":         {"enable": true, "area": [[10,11],[12,13]]},
                "detectorName": "14",
                "feedRate":     0.2,
                "duration":     15,
                "useSubStream": true
            }
        });

        let got = TfliteConfig::parse(raw).unwrap().unwrap();

        let want = TfliteConfig {
            thresholds: HashMap::from([(
                "5".to_owned().try_into().unwrap(),
                6.try_into().unwrap(),
            )]),
            crop: Crop {
                x: 7.try_into().unwrap(),
                y: 8.try_into().unwrap(),
                size: 9.try_into().unwrap(),
            },
            mask: Mask {
                enable: true,
                area: vec![
                    PointNormalized { x: 10, y: 11 },
                    PointNormalized { x: 12, y: 13 },
                ],
            },
            detector_name: "14".to_owned().try_into().unwrap(),
            feed_rate: FeedRateSec::new(Duration::from_secs(5)),
            duration: DurationSec::new(Duration::from_secs(15)),
            use_sub_stream: true,
        };
        assert_eq!(want, got);
    }

    #[test]
    fn test_parse_config_empty() {
        let raw = serde_json::Value::String(String::new());
        assert!(TfliteConfig::parse(raw).unwrap().is_none());
    }

    #[test]
    fn test_parse_config_empty2() {
        let raw = json!({"tflite": {}});
        assert!(TfliteConfig::parse(raw).unwrap().is_none());
    }
}
