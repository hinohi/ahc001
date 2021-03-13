import json

import optuna

from do_lambda import sampling

storage_name = 'sqlite:///sa.db'
study = optuna.create_study(
    study_name='20210314-001250-weight-params',
    storage=storage_name,
    load_if_exists=True,
)


def objective(trial: optuna.Trial) -> float:
    temp0 = trial.suggest_loguniform('temp0', 1e-2, 1.0)
    temp1 = trial.suggest_loguniform('temp1', 1e-6, 1e-2)
    slide_d_start = trial.suggest_loguniform('slide_d_start', 1.0, 2048.0)
    slide_d_end = trial.suggest_loguniform('slide_d_end', 1.0, 1024.0)
    grow_d1_start = trial.suggest_loguniform('grow_d1_start', 1.0, 1024.0)
    grow_d1_end = trial.suggest_loguniform('grow_d1_end', 1.0, 128.0)
    grow_d2_start = trial.suggest_loguniform('grow_d2_start', 1.0, 2048.0)
    grow_d2_end = trial.suggest_loguniform('grow_d2_end', 1.0, 2048.0)
    grow_d3_start = trial.suggest_loguniform('grow_d3_start', 1.0, 2048.0)
    grow_d3_end = trial.suggest_loguniform('grow_d3_end', 1.0, 2048.0)
    weight_slide_start = trial.suggest_uniform('weight_slide_start', 0.0, 1.0)
    weight_slide_end = trial.suggest_uniform('weight_slide_end', 0.0, 1.0)
    weight_d1_start = trial.suggest_uniform('weight_d1_start', 0.0, 1.0)
    weight_d1_end = trial.suggest_uniform('weight_d1_end', 0.0, 1.0)
    weight_d2_start = trial.suggest_uniform('weight_d2_start', 0.0, 1.0)
    weight_d2_end = trial.suggest_uniform('weight_d2_end', 0.0, 1.0)
    weight_d3_start = trial.suggest_uniform('weight_d3_start', 0.0, 1.0)
    weight_d3_end = trial.suggest_uniform('weight_d3_end', 0.0, 1.0)
    param = json.dumps({
        'temp0': temp0,
        'temp1': temp1,
        'slide_d_start': slide_d_start,
        'slide_d_end': slide_d_end,
        'grow_d1_start': grow_d1_start,
        'grow_d1_end': grow_d1_end,
        'grow_d2_start': grow_d2_start,
        'grow_d2_end': grow_d2_end,
        'grow_d3_start': grow_d3_start,
        'grow_d3_end': grow_d3_end,
        'weight_slide_start': weight_slide_start,
        'weight_slide_end': weight_slide_end,
        'weight_d1_start': weight_d1_start,
        'weight_d1_end': weight_d1_end,
        'weight_d2_start': weight_d2_start,
        'weight_d2_end': weight_d2_end,
        'weight_d3_start': weight_d3_start,
        'weight_d3_end': weight_d3_end,
    }, indent=None, separators=(',', ':'))
    scores = sampling(param, samples=300)
    return 1.0 - sum(scores.values()) / len(scores)


study.enqueue_trial({
    'temp0': 0.23753236719777623,
    'temp1': 9.5367431640625e-07,
    'slide_d_start': 104.08558092176376,
    'slide_d_end': 147.01079075305694,
    'grow_d1_start': 104.39359219777559,
    'grow_d1_end': 2.3165079582556958,
    'grow_d2_start': 660.2448737846898,
    'grow_d2_end': 5.668514832161116,
    'grow_d3_start': 660.2448737846898,
    'grow_d3_end': 5.668514832161116,
    'weight_slide_start': 0.0,
    'weight_slide_end': 0.1,
    'weight_d1_start': 1.0,
    'weight_d1_end': 0.1,
    'weight_d2_start': 0.1,
    'weight_d2_end': 0.5,
    'weight_d3_start': 0.0,
    'weight_d3_end': 0.1,
})
study.optimize(objective)
