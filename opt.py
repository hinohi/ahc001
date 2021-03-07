import logging
import sys
import json
import subprocess

import optuna


optuna.logging.get_logger("optuna").addHandler(logging.StreamHandler(sys.stdout))
logger = optuna.logging.get_logger('run')
study_name = "simple-mc_non-n-specified_100_2"
storage_name = "sqlite:///{}.db".format(study_name)
study = optuna.create_study(
    study_name=study_name,
    storage=storage_name,
    load_if_exists=True,
)


def run(param: str) -> float:
    results = []
    worker_num = 4
    workers = []

    def wait():
        tmp_workers = []
        for p in workers:
            try:
                code = p.wait(0.1)
            except subprocess.TimeoutExpired:
                tmp_workers.append(p)
                continue
            if code == 0:
                results.append(n)
            else:
                logger.warning(p.stderr.read().decode())
        workers[:] = tmp_workers

    for n in range(100):
        in_file = f'tools/in/{n:04}.txt'
        out_file = f'out/{n:04}.txt'
        p = subprocess.Popen(
            args=f"./target/release/simple_mc '{param}' < {in_file} > {out_file}",
            shell=True,
            stderr=subprocess.PIPE,
        )
        workers.append(p)
        while len(workers) == worker_num:
            wait()
    while workers:
        wait()

    score = 0
    for n in results:
        p = subprocess.run(
            ['./target/release/vis', f'tools/in/{n:04}.txt', f'out/{n:04}.txt'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=10.0,
        )
        if p.stderr:
            logger.warning(p.stderr.decode())
        try:
            score += int(p.stdout.decode().strip())
        except ValueError:
            logger.warning(p.stdout)
    return 1.0 - score / len(results) / 10 ** 9


def objective(trial: optuna.Trial) -> float:
    temp0 = trial.suggest_loguniform('temp0', 0.1, 10.0)
    temp1 = trial.suggest_loguniform('temp1', 0.0001, 0.1)
    move_d_max = trial.suggest_int('move_d_max', 1, 128)
    grow_d1_start = trial.suggest_loguniform('grow_d1_start', 2.0, 4096.0)
    grow_d1_end = trial.suggest_loguniform('grow_d1_end', 2.0, 4096.0)
    grow_d2_start = trial.suggest_loguniform('grow_d2_start', 2.0, 4096.0)
    grow_d2_end = trial.suggest_loguniform('grow_d2_end', 2.0, 4096.0)
    rect_move_weight = trial.suggest_uniform('rect_move_weight', 0.0, 1.0)
    rect_grow_d1_weight = trial.suggest_uniform('rect_grow_d1_weight', 0.0, 1.0)
    rect_grow_d2_weight = trial.suggest_uniform('rect_grow_d2_weight', 0.0, 1.0)
    param = json.dumps({
        'temp0': temp0,
        'temp1': temp1,
        'move_d_max': move_d_max,
        'grow_d1_start': grow_d1_start,
        'grow_d1_end': grow_d1_end,
        'grow_d2_start': grow_d2_start,
        'grow_d2_end': grow_d2_end,
        'rect_move_weight': rect_move_weight,
        'rect_grow_d1_weight': rect_grow_d1_weight,
        'rect_grow_d2_weight': rect_grow_d2_weight,
    }, indent=None, separators=(',', ':'))
    return run(param)


study.optimize(objective)
