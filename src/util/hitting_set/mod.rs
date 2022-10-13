use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use super::algorithms::difference;
use super::algorithms::intersection;

pub struct HSReductionResult {
    pub forced: Vec<u32>,
    pub reduced: Vec<Vec<u32>>,
}

pub fn reduce_hitting_set(original: &mut [Vec<u32>], max_value: u32) -> HSReductionResult {
    for set in original.iter_mut() {
        set.sort_unstable();
    }
    let mut instance = original.to_owned();
    let mut reduced;
    let mut total_forced = Vec::new();

    let mut apply_supersets = false;
    for set in &instance {
        if set.len() > 2 {
            apply_supersets = true;
            break;
        }
    }

    loop {
        reduced = false;

        reduced |= remove_equal(&mut instance);
        let (changed, result) = remove_unique(&mut instance, max_value);
        reduced |= changed;
        if let Some(mut forced) = result {
            total_forced.append(&mut forced);
        }

        if let Some(mut forced) = include_forced(&mut instance) {
            reduced = true;
            total_forced.append(&mut forced);
        }

        if reduced {
            continue;
        }

        if apply_supersets {
            reduced |= remove_supersets(&mut instance, max_value);
        }

        if !reduced {
            break;
        }
    }

    HSReductionResult {
        forced: total_forced,
        reduced: instance,
    }
}

fn remove_equal(instance: &mut Vec<Vec<u32>>) -> bool {
    let mut sets: FxHashMap<Vec<u32>, Vec<usize>> = FxHashMap::default();
    for i in 0..instance.len() {
        let set = &instance[i];
        if let Some(equals) = sets.get_mut(set) {
            equals.push(i);
        } else {
            sets.insert(instance[i].clone(), vec![]);
        }
    }

    let mut redundant = FxHashSet::default();
    for (_, equals) in sets {
        if !equals.is_empty() {
            for set in equals {
                redundant.insert(set);
            }
        }
    }

    if !redundant.is_empty() {
        let mut new_instance = Vec::new();
        for i in 0..instance.len() {
            if redundant.contains(&i) {
                continue;
            }
            new_instance.push(std::mem::take(&mut instance[i]));
        }
        *instance = new_instance;
        true
    } else {
        false
    }
}

fn remove_supersets(instance: &mut Vec<Vec<u32>>, max_value: u32) -> bool {
    let mut apply_supersets = false;
    for set in instance.iter() {
        if set.len() > 2 {
            apply_supersets = true;
            break;
        }
    }

    if !apply_supersets {
        return false;
    }

    let mut table = vec![Vec::new(); max_value as usize];
    for i in 0..instance.len() {
        for element in &instance[i] {
            table[*element as usize].push(i as u32);
        }
    }

    for list in &mut table {
        list.sort_unstable();
    }

    let mut to_remove = FxHashSet::default();

    for i in 0..instance.len() {
        let set = &instance[i];
        let first = set[0];
        let mut supersets = table[first as usize].clone();
        for j in 1..set.len() {
            let k = set[j];
            supersets = intersection(&supersets, &table[k as usize]);
        }
        // set is always contained in supersets
        if supersets.len() > 1 {
            for s in supersets {
                if s != i as u32 {
                    to_remove.insert(s as usize);
                }
            }
        }
    }

    let reduced = !to_remove.is_empty();
    if reduced {
        let mut new_instance = Vec::new();
        for i in 0..instance.len() {
            if to_remove.contains(&i) {
                continue;
            }
            new_instance.push(std::mem::take(&mut instance[i]));
        }
        *instance = new_instance;
    }
    reduced
}

fn remove_unique(instance: &mut Vec<Vec<u32>>, max_value: u32) -> (bool, Option<Vec<u32>>) {
    let mut count = vec![0; max_value as usize];
    for i in 0..instance.len() {
        let set = &instance[i];
        for element in set {
            if set.len() == 1 {
                count[*element as usize] += 2;
            } else {
                count[*element as usize] += 1;
            }
        }
    }

    let mut to_remove = Vec::new();
    for i in 0..count.len() {
        if count[i] == 1 {
            to_remove.push(i as u32);
        }
    }

    let result = !to_remove.is_empty();
    if result {
        let mut new_instance = Vec::new();
        let mut forced = Vec::new();
        for i in 0..instance.len() {
            let new_set = difference(&instance[i], &to_remove);
            if new_set.is_empty() {
                // The set contains only variables that appear once, we can
                // pick any variable to include.
                forced.push(instance[i][0]);
            } else {
                new_instance.push(new_set);
            }
        }
        *instance = new_instance;
        let result = if forced.is_empty() {
            None
        } else {
            Some(forced)
        };
        (true, result)
    } else {
        (false, None)
    }
}

fn include_forced(instance: &mut Vec<Vec<u32>>) -> Option<Vec<u32>> {
    let mut forced_set = FxHashSet::default();
    let mut changed = false;
    for i in 0..instance.len() {
        let set = &instance[i];
        if set.len() == 1 {
            forced_set.insert(set[0]);
            changed = true;
        }
    }

    let mut to_remove: Vec<_> = forced_set.into_iter().collect();
    to_remove.sort_unstable();

    if changed {
        let mut new_instance = Vec::with_capacity(instance.len());
        for i in 0..instance.len() {
            if instance.len() == 1 {
                continue;
            }
            let new_set = difference(&instance[i], &to_remove);
            if new_set.len() == instance[i].len() {
                new_instance.push(new_set);
            }
        }
        *instance = new_instance;
        Some(to_remove)
    } else {
        None
    }
}
