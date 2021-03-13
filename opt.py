import json
from functools import cache

import optuna
from do_lambda import sampling

storage_name = 'sqlite:///sa.db'
study = optuna.create_study(
    study_name=f'20210313-102720-500ms-judge',
    storage=storage_name,
    load_if_exists=True,
)


@cache
def get_nigate_samples() -> list[int]:
    samples = []
    for line in open('scores.txt'):
        seed, score = line.split()
        samples.append([float(score), int(seed)])
    samples.sort()
    return [s[1] for s in samples[:100]]


@cache
def get_futsu_samples() -> list[int]:
    samples = []
    for line in open('scores.txt'):
        seed, score = line.split()
        samples.append([float(score), int(seed)])
    samples.sort()
    return [s[1] for s in samples[450:550]]


@cache
def get_tokui_samples() -> list[int]:
    samples = []
    for line in open('scores.txt'):
        seed, score = line.split()
        samples.append([float(score), int(seed)])
    samples.sort(reverse=True)
    return [s[1] for s in samples[:100]]


def objective(trial: optuna.Trial) -> float:
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
    scores = sampling(param, samples=300)
    return 1.0 - sum(scores.values()) / len(scores)


study.enqueue_trial({
    'temp0': 0.38615398776136467,
    'slide_d_start': 529.3667629196551,
    'slide_d_end': 373.40914222805014,
    'grow_d1_start': 1082.4154098146191,
    'grow_d1_end': 267.86998139178905,
    'grow_d2_start': 1361.8315116712424,
    'grow_d2_end': 81.24643884887297,
    'rect_grow_d1_weight': 0.4706187875806971,
    'rect_slide_weight': 0.009042981044568259,
})
study.enqueue_trial({
    'temp0': 0.4752020144854761,
    'slide_d_start': 562.9193544814267,
    'slide_d_end': 26.33332527645415,
    'grow_d1_start': 624.3224449824461,
    'grow_d1_end': 3.44187622623260134,
    'grow_d2_start': 712.9458825147167,
    'grow_d2_end': 2.805987808579374,
    'rect_grow_d1_weight': 0.20405140603454594,
    'rect_slide_weight': 0.059358074889068135,
})
study.enqueue_trial({
    'temp0': 0.35509891828924367,
    'slide_d_start': 265.08009483681815,
    'slide_d_end': 22.038890308929503,
    'grow_d1_start': 624.3224449824461,
    'grow_d1_end': 3.44187622623260134,
    'grow_d2_start': 17.046627870003714,
    'grow_d2_end': 170.85435309175918,
    'rect_grow_d1_weight': 0.7724869554305501,
    'rect_slide_weight': 0.06434703895540622,
})
study.enqueue_trial({
    'temp0': 0.433354411088387,
    'slide_d_start': 250.5903267072716,
    'slide_d_end': 2.840762754583838,
    'grow_d1_start': 1027.3826819512265,
    'grow_d1_end': 18.498773169976594,
    'grow_d2_start': 400.83243234821305,
    'grow_d2_end': 1.8706891858113546,
    'rect_grow_d1_weight': 0.3753172948438983,
    'rect_slide_weight': 0.19026513295873007,
})
study.optimize(objective)
