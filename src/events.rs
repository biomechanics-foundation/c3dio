use crate::parameters::{Parameter, ParameterData, Parameters};
use crate::processor::Processor;
use crate::{C3dIoError, C3dParseError};
use grid::Grid;
use std::collections::HashMap;
/// # Events
///
/// Events are time points in the C3D file that are marked with a label.
/// The label is a 4-character string that can be used to identify the event.
/// The label is optional, and if it is not present, the event is still marked
/// with a time point.
/// The events are stored in the C3D file header.

/// The `Events` struct contains the events from the C3D file header.
/// Event information can be included in the parameter section of the C3D file
/// as well, that information is stored in the `Parameter` struct.
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventContext {
    pub used: Option<i16>,
    pub icon_ids: Option<Vec<u16>>,
    pub labels: Option<Vec<String>>,
    pub descriptions: Option<Vec<String>>,
    pub colours: Option<Vec<[u8; 3]>>,
}

impl EventContext {
    pub fn new() -> Self {
        EventContext::default()
    }

    pub(crate) fn from_parameters(parameters: &mut Parameters) -> Result<Self, C3dParseError> {
        let used = parameters.remove("EVENT_CONTEXT", "USED");
        let used = match used {
            Some(parameter) => Some(parameter.as_ref().try_into()?),
            _ => None,
        };
        let icon_ids = parameters.remove("EVENT_CONTEXT", "ICON_IDS");
        let icon_ids = match icon_ids {
            Some(parameter) => Some(parameter.as_ref().try_into()?),
            _ => None,
        };
        let labels = parameters.remove("EVENT_CONTEXT", "LABELS");
        let labels = match labels {
            Some(parameter) => Some(parameter.as_ref().try_into()?),
            _ => None,
        };
        let descriptions = parameters.remove("EVENT_CONTEXT", "DESCRIPTIONS");
        let descriptions = match descriptions {
            Some(parameter) => Some(parameter.as_ref().try_into()?),
            _ => None,
        };
        Ok(EventContext {
            used,
            icon_ids,
            labels,
            descriptions,
            colours: get_colour_array(parameters, "EVENT_CONTEXT", "COLOURS"),
        })
    }
}

fn get_colour_array(
    parameters: &mut Parameters,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<[u8; 3]>> {
    let parameter = parameters.remove(group_name, parameter_name)?;
    match &parameter.data {
        ParameterData::Byte(data) => {
            if parameter.dimensions.len() == 2 {
                let mut colours = Vec::new();
                for row in 0..data.len() % 3 {
                    let mut colour = [0; 3];
                    colour[0] = data[row * 3];
                    colour[1] = data[row * 3 + 1];
                    colour[2] = data[row * 3 + 2];
                    colours.push(colour);
                }
                Some(colours)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// The `Events` struct contains the events from the C3D file header.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Events {
    pub supports_events_labels: bool,
    events: Vec<Event>,
    event_context: EventContext,
}

impl ToString for Events {
    fn to_string(&self) -> String {
        let mut string = String::new();
        string.push_str("Events:\n");
        string.push_str(&format!(
            "  Supports events labels: {}\n",
            self.supports_events_labels
        ));
        string.push_str(&format!("  Number of events: {}\n", self.events.len()));
        for event in &self.events {
            string.push_str(&format!(
                "  Event: {}\n",
                event.id.iter().collect::<String>()
            ));
            string.push_str(&format!("    Label: {}\n", event.label));
            string.push_str(&format!("    Display flag: {}\n", event.display_flag));
            string.push_str(&format!("    Time: {}\n", event.time));
            string.push_str(&format!("    Context: {}\n", event.context));
            string.push_str(&format!("    Description: {}\n", event.description));
            string.push_str(&format!("    Subject: {}\n", event.subject));
            string.push_str(&format!("    Icon ID: {}\n", event.icon_id));
            string.push_str(&format!("    Generic flag: {}\n", event.generic_flag));
        }
        string
    }
}

impl Deref for Events {
    type Target = Vec<Event>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl DerefMut for Events {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}

/// The `Event` struct contains the information for a single event.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Event {
    pub id: [char; 4], // found in header
    pub label: String, // found in parameter section
    pub display_flag: bool,
    pub time: f32,
    pub context: String,
    pub description: String,
    pub subject: String,
    pub icon_id: i16,
    pub generic_flag: i16,
}

#[allow(dead_code)]
impl Event {
    pub fn new() -> Event {
        Event::default()
    }
}

impl Events {
    #[allow(dead_code)]
    pub(crate) fn new() -> Events {
        Events::default()
    }

    /// Returns the number of events in the C3D file header.
    /// The maximum number of events is 18.
    pub fn num_events(&self) -> usize {
        self.events.len()
    }

    /// Returns the event at the specified index.
    /// The index must be less than the number of events.
    /// The maximum number of events is 18.
    pub fn event(&self, index: usize) -> Option<&Event> {
        if index < self.events.len() {
            Some(&self.events[index])
        } else {
            None
        }
    }

    pub(crate) fn from_header_and_parameters(
        header_block: &[u8; 512],
        parameters: &mut Parameters,
        processor: &Processor,
    ) -> Result<Events, C3dParseError> {
        let supports_events_labels =
            processor.u16([header_block[298], header_block[299]]) == 0x3039;
        let num_time_events =
            get_num_time_events(header_block, parameters, &processor, supports_events_labels)?;

        let mut events = Vec::<Event>::with_capacity(num_time_events);

        let times = get_times_array(parameters)?;
        let labels = get_labels_array(parameters)?;
        let contexts = get_contexts_array(parameters);
        let descriptions = get_descriptions_array(parameters);
        let subjects = get_subjects_array(parameters);
        let icon_ids = get_icon_ids_array(parameters);
        let generic_flags = get_generic_flags_array(parameters);

        // event times start at byte 306
        for event_num in 0..num_time_events {
            let id = get_event_id(event_num, header_block)?;
            let label = match supports_events_labels && labels.len() > event_num {
                true => labels[event_num].clone(),
                false => "".to_string(),
            };
            let display_flag = match supports_events_labels {
                true => get_display_flag(event_num, header_block),
                false => true,
            };
            let time = verify_time(event_num, header_block, &times, processor)?;
            let context = get_event_context(event_num, &contexts);
            let description = get_event_description(event_num, &descriptions);
            let subject = get_event_subject(event_num, &subjects);
            let icon_id = get_event_icon_id(event_num, &icon_ids);
            let generic_flag = get_event_generic_flag(event_num, &generic_flags);
            events.push(Event {
                id,
                label,
                display_flag,
                time,
                context,
                description,
                subject,
                icon_id,
                generic_flag,
            });
        }
        Ok(Events {
            supports_events_labels,
            events,
            event_context: EventContext::from_parameters(parameters)?,
        })
    }

    pub(crate) fn write(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
    ) -> Result<Vec<u8>, C3dIoError> {
        let mut bytes = Vec::new();
        bytes.extend(Parameter::integer(self.events.len() as i16).write(
            processor,
            "USED".to_string(),
            group_names_to_ids["EVENT"],
            false,
        )?);
        let times = self
            .events
            .iter()
            .map(|event| event.time)
            .collect::<Vec<f32>>();
        if times.len() > 0 {
            bytes.extend(Parameter::floats(times)?.write(
                processor,
                "TIMES".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let labels = self
            .events
            .iter()
            .map(|event| event.label.clone())
            .collect::<Vec<String>>();
        if labels.len() > 0 {
            bytes.extend(Parameter::strings(labels).write(
                processor,
                "LABELS".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let contexts = self
            .events
            .iter()
            .map(|event| event.context.clone())
            .collect::<Vec<String>>();
        if contexts.len() > 0 {
            bytes.extend(Parameter::strings(contexts).write(
                processor,
                "CONTEXTS".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let descriptions = self
            .events
            .iter()
            .map(|event| event.description.clone())
            .collect::<Vec<String>>();
        if descriptions.len() > 0 {
            bytes.extend(Parameter::strings(descriptions).write(
                processor,
                "DESCRIPTIONS".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let subjects = self
            .events
            .iter()
            .map(|event| event.subject.clone())
            .collect::<Vec<String>>();
        if subjects.len() > 0 {
            bytes.extend(Parameter::strings(subjects).write(
                processor,
                "SUBJECTS".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let icon_ids = self
            .events
            .iter()
            .map(|event| event.icon_id)
            .collect::<Vec<i16>>();
        if icon_ids.len() > 0 {
            bytes.extend(Parameter::integers(icon_ids)?.write(
                processor,
                "ICON_IDS".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let generic_flags = self
            .events
            .iter()
            .map(|event| event.generic_flag)
            .collect::<Vec<i16>>();
        if generic_flags.len() > 0 {
            bytes.extend(Parameter::integers(generic_flags)?.write(
                processor,
                "GENERIC_FLAGS".to_string(),
                group_names_to_ids["EVENT"],
                false,
            )?);
        }
        let event_context_used = match &self.event_context.used {
            Some(used) => *used,
            _ => 0,
        };
        if event_context_used > 0 {
            bytes.extend(Parameter::integer(event_context_used).write(
                processor,
                "USED".to_string(),
                group_names_to_ids["EVENT_CONTEXT"],
                false,
            )?);
        }
        let event_context_icon_ids = match &self.event_context.icon_ids {
            Some(icon_ids) => icon_ids.iter().map(|id| *id as i16).collect::<Vec<i16>>(),
            _ => Vec::new(),
        };
        if event_context_icon_ids.len() > 0 {
            bytes.extend(Parameter::integers(event_context_icon_ids)?.write(
                processor,
                "ICON_IDS".to_string(),
                group_names_to_ids["EVENT_CONTEXT"],
                false,
            )?);
        }
        let event_context_labels = match &self.event_context.labels {
            Some(labels) => labels.clone(),
            _ => Vec::new(),
        };
        if event_context_labels.len() > 0 {
            bytes.extend(Parameter::strings(event_context_labels).write(
                processor,
                "LABELS".to_string(),
                group_names_to_ids["EVENT_CONTEXT"],
                false,
            )?);
        }
        let event_context_descriptions = match &self.event_context.descriptions {
            Some(descriptions) => descriptions.clone(),
            _ => Vec::new(),
        };
        if event_context_descriptions.len() > 0 {
            bytes.extend(Parameter::strings(event_context_descriptions).write(
                processor,
                "DESCRIPTIONS".to_string(),
                group_names_to_ids["EVENT_CONTEXT"],
                false,
            )?);
        }
        let event_context_colours = match &self.event_context.colours {
            Some(colours) => colours.clone(),
            _ => Vec::new(),
        };
        if event_context_colours.len() > 0 {
            let mut colours_grid = Grid::new(0, 3);
            for colour in event_context_colours {
                colours_grid.push_row(colour.to_vec());
            }
            bytes.extend(Parameter::byte_grid(colours_grid).write(
                processor,
                "COLOURS".to_string(),
                group_names_to_ids["EVENT_CONTEXT"],
                false,
            )?);
        }
        Ok(bytes)
    }
}

fn get_num_time_events(
    header_block: &[u8; 512],
    parameters: &mut Parameters,
    processor: &Processor,
    supports_events_labels: bool,
) -> Result<usize, C3dParseError> {
    match supports_events_labels {
        true => {
            let num_time_events = processor.i16([header_block[300], header_block[301]]);

            if num_time_events > 18 {
                return Err(C3dParseError::TooManyEvents(num_time_events));
            }
            let parameter_num_time_events = parameters.remove("EVENT", "USED");
            if parameter_num_time_events.is_some() {
                let parameter_num_time_events: i16 =
                    parameter_num_time_events.unwrap().as_ref().try_into()?;
                if parameter_num_time_events != num_time_events as i16 {
                    return Ok(parameter_num_time_events as usize);
                } else {
                    return Ok(num_time_events as usize);
                }
            }
        }
        false => {
            let parameter_num_time_events = parameters.remove("EVENT", "USED");
            if parameter_num_time_events.is_some() {
                let parameter_num_time_events: i16 =
                    parameter_num_time_events.unwrap().as_ref().try_into()?;
                return Ok(parameter_num_time_events as usize);
            }
        }
    }
    Ok(0)
}

fn get_times_array(parameters: &mut Parameters) -> Result<Vec<[f32; 2]>, C3dParseError> {
    let parameter = parameters.remove("EVENT", "TIMES");
    if parameter.is_none() {
        Ok(Vec::new())
    } else {
        let parameter = parameter.unwrap();
        match &parameter.data {
            ParameterData::Float(data) => {
                if parameter.dimensions.len() == 2 && data.len() > 1 {
                    let mut times = Vec::new();
                    for row in 0..data.len() % 2 {
                        let mut time = [0.0; 2];
                        time[0] = data[row * 2];
                        time[1] = data[row * 2 + 1];
                        times.push(time);
                    }
                    Ok(times)
                } else {
                    Ok(Vec::new())
                }
            }
            _ => Err(C3dParseError::InvalidParameterType(
                "EVENT ".to_string() + "TIMES",
            )),
        }
    }
}

fn get_labels_array(parameters: &mut Parameters) -> Result<Vec<String>, C3dParseError> {
    let labels = parameters.remove("EVENT", "LABELS");
    if labels.is_none() {
        Ok(Vec::new())
    } else {
        let labels = labels.unwrap();
        Ok(labels.as_ref().try_into()?)
    }
}

fn get_contexts_array(parameters: &mut Parameters) -> Vec<String> {
    let contexts = parameters.remove("EVENT", "CONTEXTS");
    if contexts.is_none() {
        Vec::new()
    } else {
        let contexts = contexts.unwrap();
        contexts.as_ref().try_into().unwrap_or(Vec::new())
    }
}

fn get_descriptions_array(parameters: &mut Parameters) -> Vec<String> {
    let descriptions = parameters.remove("EVENT", "DESCRIPTIONS");
    if descriptions.is_none() {
        Vec::new()
    } else {
        let descriptions = descriptions.unwrap();
        descriptions.as_ref().try_into().unwrap_or(Vec::new())
    }
}

fn get_subjects_array(parameters: &mut Parameters) -> Vec<String> {
    let subjects = parameters.remove("EVENT", "SUBJECTS");
    if subjects.is_none() {
        Vec::new()
    } else {
        let subjects = subjects.unwrap();
        subjects.as_ref().try_into().unwrap_or(Vec::new())
    }
}

fn get_icon_ids_array(parameters: &mut Parameters) -> Vec<i16> {
    let icon_ids = parameters.remove("EVENT", "ICON_IDS");
    if icon_ids.is_none() {
        Vec::new()
    } else {
        let icon_ids = icon_ids.unwrap();
        icon_ids.as_ref().try_into().unwrap_or(Vec::new())
    }
}

fn get_generic_flags_array(parameters: &mut Parameters) -> Vec<i16> {
    let generic_flags = parameters.remove("EVENT", "GENERIC_FLAGS");
    if generic_flags.is_none() {
        Vec::new()
    } else {
        let generic_flags = generic_flags.unwrap();
        generic_flags.as_ref().try_into().unwrap_or(Vec::new())
    }
}

fn verify_time(
    event_num: usize,
    header_block: &[u8; 512],
    _time: &Vec<[f32; 2]>,
    processor: &Processor,
) -> Result<f32, C3dParseError> {
    let time_start = 304 + (event_num * 4);
    Ok(processor.f32(header_block[time_start..time_start + 4].try_into().unwrap()))
    // TODO: use time
}

fn get_event_id(event_num: usize, header_block: &[u8; 512]) -> Result<[char; 4], C3dParseError> {
    if event_num > 18 {
        return Ok([0x00 as char; 4]);
    }
    let label_start = 396 + (event_num * 4);
    let label_bytes: [u8; 4] = header_block[label_start..label_start + 4]
        .try_into()
        .unwrap();
    let label_chars = label_bytes
        .iter()
        .map(|b| *b as char)
        .collect::<Vec<char>>();
    Ok(label_chars.try_into().unwrap())
}

fn get_display_flag(event_num: usize, header_block: &[u8; 512]) -> bool {
    let display_flag_start = 376 + event_num;
    header_block[display_flag_start] == 0
}

fn get_event_context(event_num: usize, contexts: &Vec<String>) -> String {
    if contexts.len() <= event_num {
        return "".to_string();
    }
    contexts[event_num].clone()
}

fn get_event_description(event_num: usize, descriptions: &Vec<String>) -> String {
    if descriptions.len() <= event_num {
        return "".to_string();
    }
    descriptions[event_num].clone()
}

fn get_event_subject(event_num: usize, subjects: &Vec<String>) -> String {
    if subjects.len() <= event_num {
        return "".to_string();
    }
    subjects[event_num].clone()
}

fn get_event_icon_id(event_num: usize, icon_ids: &Vec<i16>) -> i16 {
    if icon_ids.len() <= event_num {
        return 0;
    }
    icon_ids[event_num]
}

fn get_event_generic_flag(event_num: usize, generic_flags: &Vec<i16>) -> i16 {
    if generic_flags.len() <= event_num {
        return 0;
    }
    generic_flags[event_num]
}
