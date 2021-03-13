import json
from functools import cache

import optuna
from do_lambda import sampling

storage_name = 'sqlite:///sa.db'
study = optuna.create_study(
    study_name=f'20210313-121042-4450ms-futsu2',
    storage=storage_name,
    load_if_exists=True,
)


@cache
def get_nigate_samples() -> list[int]:
    samples = []
    for line in open('scores_500ms.txt'):
        seed, score = line.split()
        samples.append([float(score), int(seed)])
    samples.sort()
    return [s[1] for s in samples[:100]]


@cache
def get_futsu_samples() -> list[int]:
    samples = []
    for line in open('scores_500ms.txt'):
        seed, score = line.split()
        samples.append([float(score), int(seed)])
    samples.sort()
    return [s[1] for s in samples[450:550]]


@cache
def get_tokui_samples() -> list[int]:
    samples = []
    for line in open('scores_500ms.txt'):
        seed, score = line.split()
        samples.append([float(score), int(seed)])
    samples.sort(reverse=True)
    return [s[1] for s in samples[:100]]


def objective(trial: optuna.Trial) -> float:
    temp0 = trial.suggest_uniform('temp0', 0.01, 2.0)
    temp1 = trial.suggest_loguniform('temp1', 2 ** -20, 0.01)
    slide_d_start = trial.suggest_loguniform('slide_d_start', 1.0, 4096.0)
    slide_d_end = trial.suggest_loguniform('slide_d_end', 1.0, 4096.0)
    grow_d1_start = trial.suggest_loguniform('grow_d1_start', 1.0, 4096.0)
    grow_d1_end = trial.suggest_loguniform('grow_d1_end', 1.0, 4096.0)
    grow_d2_start = trial.suggest_loguniform('grow_d2_start', 1.0, 4096.0)
    grow_d2_end = trial.suggest_loguniform('grow_d2_end', 1.0, 4096.0)
    rect_grow_d1_weight = trial.suggest_uniform('rect_grow_d1_weight', 0.0, 1.0)
    rect_slide_weight = trial.suggest_uniform('rect_slide_weight', 0.0, 0.5)
    param = json.dumps({
        'temp0': temp0,
        'temp1': temp1,
        'slide_d_start': slide_d_start,
        'slide_d_end': slide_d_end,
        'grow_d1_start': grow_d1_start,
        'grow_d1_end': grow_d1_end,
        'grow_d2_start': grow_d2_start,
        'grow_d2_end': grow_d2_end,
        'rect_grow_d1_weight': rect_grow_d1_weight,
        'rect_slide_weight': rect_slide_weight,
    }, indent=None, separators=(',', ':'))
    scores = sampling(param, samples=get_futsu_samples())
    return 1.0 - sum(scores.values()) / len(scores)


study.optimize(objective)
