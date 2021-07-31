mod ds1000e;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Waveform data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WaveformData {
    pub header: WaveformHeader,
    pub data: RawData,
}

/// Waveform header
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WaveformHeader {
    pub adc_mode: u8,
    pub roll_stop: u32,
    pub active_channel: u8,
    pub ch1: ChannelHeader,
    pub ch2: ChannelHeader,
    pub time: TimeHeader,
    pub time2: TimeHeader,
    pub trigger1: TriggerHeader,
    pub trigger2: TriggerHeader,
    pub logic: LogicAnalyzerHeader,
    pub ch1_points: u32,
    pub ch1_skip: u32,
    pub ch2_points: u32,
}

/// Channel header
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChannelHeader {
    pub scale_display: i32,
    pub shift_display: i16,
    pub probe_value: f32,
    pub invert_display: u8,
    pub scale_measured: i32,
    pub shift_measured: i16,
    pub inverted: bool,
    pub enabled: bool,
    pub volt_per_division: f32,
    pub volt_scale: f32,
    pub volt_offset: f32,
    pub unit: Unit,
}

/// Time header
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeHeader {
    pub scale_display: i64,
    pub offset_display: i64,
    pub sample_rate_hz: f32,
    pub scale_measured: i64,
    pub offset_measured: i64,
}

/// Trigger header
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TriggerHeader {
    pub mode: TriggerMode,
    pub source: Source,
    pub coupling: Coupling,
    pub sweep: u8, // TODO:
    pub sens: f32,
    pub holdoff: f32,
    pub level: f32,
    pub direct: bool,
    pub pulse_type: u8, // TODO:
    pub pulse_width: f32,
    pub slope_type: u8, // TODO:
    pub lower: f32,
    pub slope_width: f32,
    pub video_pol: u8,  // TODO:
    pub video_sync: u8, // TODO:
    pub video_std: u8,  // TODO:
}

/// Logic Analyzer header
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LogicAnalyzerHeader {
    pub enabled: bool,
    pub active_channel: u8,
    pub enabled_channels: u16,
    pub position: [u8; 16], // TODO:
    pub group8to15size: u8, // TODO:
    pub group0to7size: u8,  // TODO:
}

/// Raw data
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RawData {
    pub ch1: Vec<u8>,
    pub ch2: Vec<u8>,
    pub logic: Vec<u16>,
}

/// Bandwidth
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Bandwidth {
    NoLimit = 0,
    Mhz20 = 1,
    Mhz100 = 2,
    Mhz200 = 3,
    Mhz250 = 4,
}

/// Coupling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Coupling {
    Dc = 0,
    Ac = 1,
    Gnd = 2,
}

/// Filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Filter {
    LowPass = 0,
    HighPass = 1,
    BandPass = 2,
    BandReject = 3,
}

/// Source
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Source {
    Ch1 = 0,
    Ch2 = 1,
    Ext = 2,
    Ext5 = 3,
    AcLine = 4,
    DigCh = 5,
}

/// Trigger mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum TriggerMode {
    Edge = 0,
    Pulse = 1,
    Slope = 2,
    Video = 3,
    Alt = 4,
    Pattern = 5,
    Duration = 6,
}

/// Unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Unit {
    W = 0,
    A = 1,
    V = 2,
    U = 3,
}

macro_rules! try_from_num {
    ( $( $type:ty: $max:literal, )* ) => {
        $(
            impl core::convert::TryFrom<u8> for $type {
                type Error = ();

                fn try_from(raw: u8) -> Result<Self, Self::Error> {
                    if raw <= $max {
                        Ok(unsafe { core::mem::transmute(raw) })
                    } else {
                        Err(())
                    }
                }
            }
        )*
    };
}

try_from_num! {
    Bandwidth: 4,
    Coupling: 2,
    Filter: 3,
    Source: 5,
    TriggerMode: 6,
    Unit: 3,
}
