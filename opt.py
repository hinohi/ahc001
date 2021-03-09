import json
import time
import uuid
import sys

import boto3
import optuna

N = int(sys.argv[1])

QUEUE_URL = 'https://sqs.ap-northeast-1.amazonaws.com/169698630369/ahc001-Queue4A7E3555-RODR3OR8LUGQ'

storage_name = 'sqlite:///sa.db'
study = optuna.create_study(
    study_name=f'sa-{N}',
    storage=storage_name,
    load_if_exists=True,
)

lambda_client = boto3.client('lambda')
sqs_client = boto3.client('sqs')


def invoke(arg: str) -> set[str]:
    message_ids = set()
    for seed in range(100):
        message_id = str(uuid.uuid4())
        lambda_client.invoke(
            FunctionName='ahc001',
            InvocationType='Event',
            LogType='None',
            Payload=json.dumps({
                'message_id': message_id,
                'n': N,
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
            time.sleep(5)
    return scores


def run(arg: str) -> float:
    message_ids = invoke(arg)
    time.sleep(5)
    scores = receive(message_ids)
    return 1.0 - sum(scores) / len(scores)


def objective(trial: optuna.Trial) -> float:
    temp0 = trial.suggest_loguniform('temp0', 0.01, 1.0)
    temp1 = trial.suggest_loguniform('temp1', 1e-8, 0.01)
    grow_d1_start = trial.suggest_loguniform('grow_d1_start', 2.0, 4096.0)
    grow_d1_end = trial.suggest_loguniform('grow_d1_end', 2.0, 4096.0)
    grow_d2_start = trial.suggest_loguniform('grow_d2_start', 2.0, 4096.0)
    grow_d2_end = trial.suggest_loguniform('grow_d2_end', 2.0, 4096.0)
    rect_grow_d1_weight = trial.suggest_uniform('rect_grow_d1_weight', 0.0, 1.0)
    param = json.dumps({
        'temp0': temp0,
        'temp1': temp1,
        'grow_d1_start': grow_d1_start,
        'grow_d1_end': grow_d1_end,
        'grow_d2_start': grow_d2_start,
        'grow_d2_end': grow_d2_end,
        'rect_grow_d1_weight': rect_grow_d1_weight,
    }, indent=None, separators=(',', ':'))
    return run(param)


study.optimize(objective)
