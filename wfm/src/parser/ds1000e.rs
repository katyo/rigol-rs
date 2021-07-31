/**

Rigol DS1102E oscilloscope waveform file format parser

*/
use core::convert::TryInto;
use nom::{
    cond, count, dbg_basic, map, map_opt, named, named_args,
    number::streaming::{
        le_f32 as f32, le_i16 as i16, le_i32 as i32, le_i64 as i64, le_u16 as u16, le_u32 as u32,
        u8,
    },
    tag, take, tuple,
};

use super::{
    ChannelHeader, LogicAnalyzerHeader, RawData, TimeHeader, TriggerHeader, TriggerMode, Unit,
    WaveformData, WaveformHeader,
};

pub fn parse(input: &[u8]) -> Result<WaveformData, String> {
    let header = waveform_header(input)
        .map_err(|error| format!("Unable to parse header at: {}", error))?
        .1;

    let data = raw_data(&input[276..], &header)
        .map_err(|error| format!("Unable to parse raw data: {}", error))?
        .1;

    Ok(WaveformData { header, data })
}

named!(
    waveform_header<WaveformHeader>,
    map_opt!(
        tuple!(
            tag!([0xa5u8, 0xa5, 0x00, 0x00]), // magic
            take!(12),                        // padding
            u8,                               // ADC mode
            take!(3),                         // padding
            u32,                              // roll_stop
            take!(4),                         // unused
            u32,                              // ch1 points
            u8,                               // active channel
            take!(1),                         // padding
            channel_header,                   // channel 1 header
            channel_header,                   // channel 2 header
            u8,                               // time offset
            take!(1),                         // padding
            time_header,                      // time header
            logic_analyzer_header,            // logic analyzer header
            u8,                               // trigger mode
            trigger_header,                   // trigger 1 header
            trigger_header,                   // trigger 2 header
            take!(6),                         // padding
            u32,                              // ch2 points
            time_header,                      // time 2 header
            f32                               // logic sample rate
        ),
        |(
            _magic,
            _,
            adc_mode,
            _,
            roll_stop,
            _,
            ch1_points,
            active_channel,
            _,
            ch1,
            ch2,
            time_offset,
            _,
            time,
            logic,
            trigger_mode,
            trigger1,
            trigger2,
            _,
            ch2_points,
            time2,
            logic_sample_rate,
        ): (
            &[u8],
            &[u8],
            u8,
            &[u8],
            u32,
            &[u8],
            u32,
            u8,
            &[u8],
            ChannelHeader,
            ChannelHeader,
            u8,
            &[u8],
            TimeHeader,
            LogicAnalyzerHeader,
            u8,
            TriggerHeader,
            TriggerHeader,
            &[u8],
            u32,
            TimeHeader,
            f32
        )| {
            // In rolling mode, change the number of valid samples and skip invalid points
            let (ch1_points, ch1_skip) = if roll_stop == 0 {
                (ch1_points - 4, 0)
            } else {
                (ch1_points - roll_stop - 6, roll_stop + 2)
            };

            let sample_rate_hz = time.sample_rate_hz;
            let seconds_per_point = 1.0 / sample_rate_hz;

            // Use ch1_points when ch2_points is not written
            let ch2_points = if ch1.enabled && ch2_points == 0 {
                ch1_points
            } else {
                ch2_points
            };

            // In rolling mode, skip invalid samples
            let ch1_volt_length = ch1_points - roll_stop;
            let ch2_volt_length = ch2_points - roll_stop;

            let ch1_time_scale = 1.0e-12 * time.scale_measured as f32;
            let ch1_time_offset = 1.0e-12 * time.offset_measured as f32;

            let trigger_mode: TriggerMode = (trigger_mode as u8).try_into().ok()?;

            let ch2_time_scale = if trigger_mode == TriggerMode::Alt {
                1.0e-12 * time2.scale_measured as f32
            } else {
                ch1_time_scale
            };
            let ch2_time_offset = if trigger_mode == TriggerMode::Alt {
                1.0e-12 * time2.offset_measured as f32
            } else {
                ch1_time_offset
            };

            Some(WaveformHeader {
                adc_mode,
                roll_stop,
                active_channel,
                ch1,
                ch2,
                time,
                time2,
                trigger1,
                trigger2,
                logic,
                ch1_points,
                ch1_skip,
                ch2_points,
                //seconds_per_point,
            })
        }
    )
);

named!(
    channel_header<ChannelHeader>,
    map!(
        tuple!(
            u16, // unknown
            i32, // scale display
            i16, // shift display
            u8,  // unknown
            u8,  // unknown
            f32, // probe value
            u8,  // invert display
            u8,  // enabled
            u8,  // inverted
            u8,  // unknown
            i32, // scale measured
            i16  // shift measured
        ),
        |(
            _,
            scale_display,
            shift_display,
            _,
            _,
            probe_value,
            invert_display,
            enabled,
            inverted,
            _,
            scale_measured,
            shift_measured,
        )| {
            let inverted = inverted != 0;
            let enabled = enabled != 0;

            let scale_measured_float = scale_measured as f32;
            let shift_measured_float = shift_measured as f32;

            let volt_per_division =
                (scale_measured_float * probe_value).copysign(if inverted { -1.0 } else { 1.0 });

            let volt_scale = 1.0e-6 * scale_measured_float * probe_value / 25.0;
            let volt_offset = shift_measured_float * volt_scale;

            let unit = Unit::V;

            ChannelHeader {
                scale_display,
                shift_display,
                probe_value,
                invert_display,
                scale_measured,
                shift_measured,
                inverted,
                enabled,
                volt_per_division,
                volt_scale,
                volt_offset,
                unit,
            }
        }
    )
);

named!(
    time_header<TimeHeader>,
    map!(
        tuple!(
            i64, // scale display
            i64, // offset display
            f32, // sample rate Hz
            i64, // scale measured
            i64  // offset measured
        ),
        |(scale_display, offset_display, sample_rate_hz, scale_measured, offset_measured)| {
            TimeHeader {
                scale_display,
                offset_display,
                sample_rate_hz,
                scale_measured,
                offset_measured,
            }
        }
    )
);

named!(
    trigger_header<TriggerHeader>,
    map_opt!(
        tuple!(
            u8,       // mode
            u8,       // source
            u8,       // coupling
            u8,       // sweep
            take!(1), // padding
            f32,      // sens
            f32,      // holdoff
            f32,      // level
            u8,       // direct
            u8,       // pulse type
            take!(2), // padding
            f32,      // pulse width
            u8,       // slope type
            take!(3), // padding
            f32,      // lower
            f32,      // slope width
            u8,       // video pol
            u8,       // video sync
            u8        // video std
        ),
        |(
            mode,
            source,
            coupling,
            sweep,
            _,
            sens,
            holdoff,
            level,
            direct,
            pulse_type,
            _,
            pulse_width,
            slope_type,
            _,
            lower,
            slope_width,
            video_pol,
            video_sync,
            video_std,
        )| {
            let mode = (mode as u8).try_into().ok()?;
            let source = (source as u8).try_into().ok()?;
            let coupling = (coupling as u8).try_into().ok()?;
            let direct = direct != 0;

            Some(TriggerHeader {
                mode,
                source,
                coupling,
                sweep,
                sens,
                holdoff,
                level,
                direct,
                pulse_type,
                pulse_width,
                slope_type,
                lower,
                slope_width,
                video_pol,
                video_sync,
                video_std,
            })
        }
    )
);

named!(
    logic_analyzer_header<LogicAnalyzerHeader>,
    map!(
        tuple!(
            u8,        // enabled
            u8,        // active channel (0..16)
            u16,       // enabled channels
            take!(16), // position
            u8,        // group 8..15 size
            u8         // group 0..7 size
        ),
        |(enabled, active_channel, enabled_channels, position, group8to15size, group0to7size)| {
            let enabled = enabled & 0b1 != 0;
            let position = position.try_into().unwrap();

            LogicAnalyzerHeader {
                enabled,
                active_channel,
                enabled_channels,
                position,
                group8to15size,
                group0to7size,
            }
        }
    )
);

named_args!(
    raw_data<'a>(header: &'a WaveformHeader)<RawData>,
    map!(
        tuple!(
            cond!(header.ch1.enabled,
                  tuple!(
                      take!(header.ch1_points), // channel 1 points
                      take!(header.ch1_skip), // roll stop padding 1
                      take!(4) // sentinel between datasets
                  )
            ),
            cond!(header.ch2.enabled,
                  tuple!(
                      take!(header.ch2_points), // channel 2 points
                      take!(header.ch1_skip), // roll stop padding 2
                      take!(4) // sentinel between datasets
                  )
            ),
            // Not clear where the LA length is stored assume same as ch1_points
            cond!(header.logic.enabled,
                  count!(u16, header.ch1_points as usize)
            )
        ), |(
            ch1,
            ch2,
            logic,
        )| RawData {
            ch1: ch1.map(|(smps, _, _)| smps.into()).unwrap_or_default(),
            ch2: ch2.map(|(smps, _, _)| smps.into()).unwrap_or_default(),
            logic: logic.map(|smps| smps.into()).unwrap_or_default(),
        }
    )
);

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read;

    #[test]
    fn ds1052e_2ch() {
        let i = read("test/ds1052e_2ch.wfm").unwrap();
        let r = parse(&i).unwrap();

        //println!("{:?}", r.header);
        assert_eq!(r.header.active_channel, 1);
        assert_eq!(r.header.ch1_points, 524284);
        assert_eq!(r.data.ch1.len(), 524284);
        //assert!(false);
    }
}
