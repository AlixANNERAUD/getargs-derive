const MAX_CANDIDATE_LENGTH: usize = 64;

fn distance(input: &str, candidate: &str) -> Option<usize> {
    let length = candidate.chars().count();
    if length > MAX_CANDIDATE_LENGTH {
        return None;
    }
    let mut before_previous = [0_usize; MAX_CANDIDATE_LENGTH + 1];
    let mut previous = [0_usize; MAX_CANDIDATE_LENGTH + 1];
    let mut current = [0_usize; MAX_CANDIDATE_LENGTH + 1];
    for (index, cell) in previous.iter_mut().enumerate().take(length + 1) {
        *cell = index;
    }
    let mut previous_input_char = None;
    for (input_index, input_char) in input.chars().enumerate() {
        current[0] = input_index + 1;
        let mut previous_candidate_char = None;
        for (candidate_index, candidate_char) in candidate.chars().enumerate() {
            let column = candidate_index + 1;
            let substitution = previous[column - 1] + usize::from(input_char != candidate_char);
            let mut best = substitution
                .min(previous[column] + 1)
                .min(current[column - 1] + 1);
            if previous_input_char == Some(candidate_char)
                && previous_candidate_char == Some(input_char)
            {
                best = best.min(before_previous[column - 2] + 1);
            }
            current[column] = best;
            previous_candidate_char = Some(candidate_char);
        }
        before_previous = previous;
        previous = current;
        previous_input_char = Some(input_char);
    }
    Some(previous[length])
}

/// Returns the candidate closest to `input` by edit distance, if it is close enough to be a
/// likely typo.
#[doc(hidden)]
pub fn suggest(input: &str, candidates: &[&[&'static str]]) -> Option<&'static str> {
    let input_length = input.chars().count();
    let mut best: Option<(usize, &'static str)> = None;
    for &candidate in candidates.iter().flat_map(|group| group.iter()) {
        let Some(distance) = distance(input, candidate) else {
            continue;
        };
        let threshold = input_length.max(candidate.chars().count()) / 3;
        let improves = best.is_none_or(|(best_distance, _)| distance < best_distance);
        if distance <= threshold && improves {
            best = Some((distance, candidate));
        }
    }
    best.map(|(_, candidate)| candidate)
}
