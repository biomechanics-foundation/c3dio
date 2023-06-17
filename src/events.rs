use crate::processor::Processor;
use crate::C3dParseError;

#[derive(Debug, Clone, PartialEq)]
pub struct Events {
    pub supports_events_labels: bool,
    events: Vec<Event>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Event {
    pub label: [char; 4],
    pub display_flag: bool,
    pub time: f32,
}

impl Events {
    pub fn new() -> Events {
        Events {
            supports_events_labels: false,
            events: Vec::new(),
        }
    }

    pub fn from_header_block(
        header_block: &[u8; 512],
        processor: &Processor,
    ) -> Result<Events, C3dParseError> {
        let supports_events_labels =
            processor.u16(header_block[300..302].try_into().unwrap()) == 0x3039;
        let num_time_events = header_block[302] as usize;

        if num_time_events > 18 {
            return Err(C3dParseError::TooManyEvents(num_time_events));
        }

        let mut events = Vec::new();

        // event times start at byte 306
        for i in 0..num_time_events {
            let time_start = 304 + (i * 4);
            let label_start = 398 + (i * 4);
            let label_bytes: [u8; 4] = header_block[label_start..label_start + 4]
                .try_into()
                .unwrap();
            let label_chars = label_bytes
                .iter()
                .map(|b| *b as char)
                .collect::<Vec<char>>();
            let display_flag_start = 378 + i;
            let display_flag_byte: [u8; 1] = header_block
                [display_flag_start..display_flag_start + 1]
                .try_into()
                .unwrap();
            let display_flag = display_flag_byte[0] == 0x01;
            events.push(Event {
                time: processor.f32(header_block[time_start..time_start + 4].try_into().unwrap()),
                label: label_chars.try_into().unwrap(),
                display_flag,
            });
        }
        Ok(Events {
            supports_events_labels,
            events,
        })
    }
}
