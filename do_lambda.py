import json
import time
import uuid

import boto3

QUEUE_URL = 'https://sqs.ap-northeast-1.amazonaws.com/169698630369/ahc001-Queue4A7E3555-X7XRXLQUFTF3'

lambda_client = boto3.client('lambda')
sqs_client = boto3.client('sqs')


def invoke(arg: str = None, *, samples=None) -> dict[str, int]:
    message_seed = {}
    if samples is None:
        samples = range(100)
    elif isinstance(samples, int):
        samples = range(samples)
    for seed in samples:
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
        message_seed[message_id] = seed
    return message_seed


def receive(message_seed: dict[str, int]) -> dict[int, float]:
    limit = 300
    start = time.time()
    scores = {}
    receipt_handles = {}
    while message_seed and (time.time() - start) < limit:
        r = sqs_client.receive_message(
            QueueUrl=QUEUE_URL,
            MaxNumberOfMessages=10,
        )
        for record in r.get('Messages', []):
            rh = record['ReceiptHandle']
            receipt_handles[str(uuid.uuid4())] = rh
            body = json.loads(record['Body'])['responsePayload']
            seed = message_seed.pop(body['message_id'], None)
            if seed is None:
                continue
            scores[seed] = body['score']
        if receipt_handles:
            deleted = sqs_client.delete_message_batch(
                QueueUrl=QUEUE_URL,
                Entries=[{'Id': key, 'ReceiptHandle': rh} for key, rh in receipt_handles.items()],
            )
            for suc in deleted.get('Successful', []):
                receipt_handles.pop(suc['Id'], None)
        elif message_seed:
            time.sleep(1)
    return scores


def sampling(arg: str = None, *, samples) -> dict[int, float]:
    message_seed = invoke(arg, samples=samples)
    time.sleep(1)
    return receive(message_seed)


def main():
    scores = sampling(samples=1000)
    for seed in sorted(scores):
        print(seed, scores[seed])


if __name__ == '__main__':
    main()
