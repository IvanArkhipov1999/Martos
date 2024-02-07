use crate::timer::TickType;

/// Type for information from one timer
type TimerInformation = TickType;
/// Type for information from several timers
type TimersInformation = Vec<TimerInformation>;

/// Sends timer information to neighbours (or adds it into package to send).
pub fn send_timer_information(_timer_information: TimerInformation) {
    // TODO: Some code for sending information or adding it into package to send. It may be hardware dependent.
}

/// Gets timer information from neighbour timers.
pub fn get_timers_information() -> TimersInformation {
    // TODO: Some code, receiving information from other timers. It may be hardware dependent.
    vec![1, 2 ,3]
}