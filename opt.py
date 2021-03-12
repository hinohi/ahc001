import json

import optuna
from do_lambda import sampling

storage_name = 'sqlite:///sa.db'
study = optuna.create_study(
    study_name=f'20210312-141343-greedy-push_by',
    storage=storage_name,
    load_if_exists=True,
)


def objective(trial: optuna.Trial) -> float:
    n_try = trial.suggest_int('n_try', 1, 3)
    temp0 = trial.suggest_uniform('temp0', 0.1, 0.5)
    slide_d_start = trial.suggest_loguniform('slide_d_start', 1.0, 4096.0)
    slide_d_end = trial.suggest_loguniform('slide_d_end', 1.0, 4096.0)
    grow_d1_start = trial.suggest_loguniform('grow_d1_start', 1.0, 4096.0)
    grow_d1_end = trial.suggest_loguniform('grow_d1_end', 1.0, 4096.0)
    grow_d2_start = trial.suggest_loguniform('grow_d2_start', 1.0, 4096.0)
    grow_d2_end = trial.suggest_loguniform('grow_d2_end', 1.0, 4096.0)
    rect_grow_d1_weight = trial.suggest_uniform('rect_grow_d1_weight', 0.0, 1.0)
    rect_slide_weight = trial.suggest_uniform('rect_slide_weight', 0.0, 0.5)
    param = json.dumps({
        'n_try': n_try,
        'temp0': temp0,
        'temp1': 2.0 ** -20,
        'slide_d_start': slide_d_start,
        'slide_d_end': slide_d_end,
        'grow_d1_start': grow_d1_start,
        'grow_d1_end': grow_d1_end,
        'grow_d2_start': grow_d2_start,
        'grow_d2_end': grow_d2_end,
        'rect_grow_d1_weight': rect_grow_d1_weight,
        'rect_slide_weight': rect_slide_weight,
    }, indent=None, separators=(',', ':'))
    scores = sampling(param, samples=100)
    return 1.0 - sum(scores.values()) / len(scores)


study.optimize(objective)
