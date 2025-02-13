use rand::{distributions::Uniform, prelude::Distribution};

use crate::{
    ExperimentInstance, IcedDobotController, ResultExports, ResultItem, ShapeAnalysis, Shapes,
};

pub fn create_experiment_instance(appv: &IcedDobotController) -> ExperimentInstance {
    let n_tests = appv
        .active_experiment_parameters
        .test_count
        .value
        .parse::<usize>()
        .unwrap();

    let shapes_to_use = appv
        .active_experiment_shapes_to_use
        .clone()
        .iter()
        .filter(|(_shape, is_use)| *is_use)
        .map(|(shape, _)| shape.clone())
        .collect::<Vec<Shapes>>();

    let n_used_shapes = shapes_to_use.len();

    let mut shapes_buffer = Vec::with_capacity(n_tests);
    let between = Uniform::from(0..n_used_shapes);
    let mut rng = rand::thread_rng();

    for _ in 0..n_tests {
        shapes_buffer.push(shapes_to_use[between.sample(&mut rng)])
    }

    ExperimentInstance {
        shapes_selection: shapes_to_use.clone(),
        list_of_shapes: shapes_buffer,
        list_of_guesses: Vec::with_capacity(n_tests),
        list_of_retries: vec![0; n_tests],
        list_of_time: Vec::with_capacity(n_tests),
        experiment_index: 0,
    }
}

pub async fn process_results<'a>(
    appv: IcedDobotController,
) -> (Option<ResultExports>, Option<String>) {
    let result = appv.active_experiment_instance.clone().unwrap();
    let counts = result.list_of_guesses.capacity();

    let mut total_time = 0;
    let mut total_correct = 0;

    let mut analysis_vec = result
        .shapes_selection
        .iter()
        .map(|shape| ShapeAnalysis::new(shape.clone(), result.shapes_selection.clone()))
        .collect::<Vec<ShapeAnalysis>>();

    let result_items: Vec<ResultItem> = result
        .list_of_shapes
        .iter()
        .zip(result.list_of_guesses.iter())
        .zip(result.list_of_time.iter())
        .zip(result.list_of_retries.iter())
        .map(|(((real, guess), r_time), retries)| {
            let _ = analysis_vec
                .iter_mut()
                .filter(|shape_analysis| shape_analysis.main_shape == *real)
                .map(|shape_analysis| {
                    classify_answer(*guess, shape_analysis, *r_time);
                })
                .collect::<Vec<_>>();
            total_time += r_time;
            (real, guess, r_time, retries)
        })
        .fold(Vec::new(), |mut accu, (real, guess, r_time, retries)| {
            accu.push(ResultItem {
                time       : *r_time,
                true_shape : *real,
                guess_shape: *guess,
                retries    : *retries,
                is_correct : {
                    if real == guess {
                        total_correct += 1;
                        true
                    } else {
                        false
                    }
                },
            });
            accu
        });

    let _ = analysis_vec
        .iter_mut()
        .map(|analysis| analysis.calc_avg_time())
        .collect::<Vec<_>>();

    let final_result = ResultExports {
        parameters: appv.active_experiment_parameters.clone(),
        avg_time: (total_time / counts as u128),
        avg_correct_answers: total_correct,
        analyses: analysis_vec.clone(),
        results: result_items,
    };

    let final_string = serde_json::to_string_pretty(&final_result).unwrap();

    let save_file_name = format!(
        "N{}_T{}_V{}_A{}_S{}_MT{}_MF{}_CT{}_CF{}",
        final_result.parameters.subject_name.value,
        final_result.parameters.test_count.value,
        final_result.parameters.voltage.value,
        final_result.parameters.acceleration.value,
        final_result.parameters.speed.value,
        final_result.parameters.modulation_type.value,
        final_result.parameters.modulation_frequency.value,
        final_result.parameters.carrier_type.value,
        final_result.parameters.carrier_frequency.value,
    );

    let results_dir = &appv.active_config.results_path;

    let save_file_path = format!("{}/{}", results_dir, save_file_name);

    let mut final_path = save_file_path.clone();
    let mut ndup: u32 = 1;

    while tokio::fs::try_exists(format!("{}.json", final_path))
        .await
        .unwrap()
    {
        final_path = format!("{}_D{}", save_file_path, ndup);
        ndup += 1;
    }

    match tokio::fs::write(final_path, final_string).await {
        Ok(_) => (Some(final_result), None),
        Err(errmsg) => (None, Some(format!("{}", errmsg))),
    }
}

pub fn classify_answer(guess: Shapes, analysis: &mut ShapeAnalysis, guesstime: u128) {
    if guess == analysis.main_shape {
        analysis.main_shape_count += 1;
    } else {
        let _ = analysis
            .wrong_shapes
            .iter_mut()
            .filter(|(shape, _)| *shape == guess)
            .map(|(_, shape_count)| *shape_count += 1)
            .collect::<Vec<_>>();
    }
    analysis.time += guesstime;
}

