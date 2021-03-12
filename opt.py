import json
import time
import uuid

import boto3
import optuna

QUEUE_URL = 'https://sqs.ap-northeast-1.amazonaws.com/169698630369/ahc001-Queue4A7E3555-RODR3OR8LUGQ'

storage_name = 'sqlite:///sa.db'
study = optuna.create_study(
    study_name=f'20210312-141343-greedy-push_by',
    storage=storage_name,
    load_if_exists=True,
)

lambda_client = boto3.client('lambda')
sqs_client = boto3.client('sqs')


def invoke(arg: str) -> set[str]:
    message_ids = set()
    for seed in range(300, 400):
        message_id = str(uuid.uuid4())
        lambda_client.invoke(
            FunctionName='ahc001',
            InvocationType='Event',
            LogType='None',
            Payload=json.dumps({
                'message_id': message_id,
                'seed': seed,
                'arg': arg,
            }),
        )
        message_ids.add(message_id)
    return message_ids


def receive(message_ids: set[str]) -> list[float]:
    scores = []
    receipt_handles = {}
    while message_ids:
        r = sqs_client.receive_message(
            QueueUrl=QUEUE_URL,
            MaxNumberOfMessages=10,
        )
        for record in r.get('Messages', []):
            rh = record['ReceiptHandle']
            receipt_handles[str(uuid.uuid4())] = rh
            body = json.loads(record['Body'])['responsePayload']
            try:
                message_ids.remove(body['message_id'])
            except KeyError:
                continue
            scores.append(body['score'])
        if receipt_handles:
            deleted = sqs_client.delete_message_batch(
                QueueUrl=QUEUE_URL,
                Entries=[{'Id': key, 'ReceiptHandle': rh} for key, rh in receipt_handles.items()],
            )
            for suc in deleted.get('Successful', []):
                receipt_handles.pop(suc['Id'], None)
        elif message_ids:
            time.sleep(3)
    return scores


def run(arg: str) -> float:
    message_ids = invoke(arg)
    time.sleep(5)
    scores = receive(message_ids)
    return 1.0 - sum(scores) / len(scores)


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
    return run(param)


study.optimize(objective)
