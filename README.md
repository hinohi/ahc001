# ahc001

## AtCoder 本番のめも

### ライブラリ

https://github.com/rust-lang-ja/atcoder-rust-resources/wiki/2020-Update

### 実行速度

`in/0001.txt` をインプットに、limit=1000ms で。

手元

```
$ ./target/release/local > /dev/null
Count { all: 23393000, tried: 23393000, valid: 12532383, ac: 900411 }
```

AtCoder コードテスト

終了コード	0
実行時間	1006 ms
メモリ	400 KB

```
Count { all: 15298000, tried: 15298000, valid: 8324467, ac: 577954 }
```

Lambda 1024MB

```
Count { all: 9059000, tried: 9059000, valid: 4894412, ac: 384759 }
```

Lambda 1530MB

```
Count { all: 13655000, tried: 13655000, valid: 7264307, ac: 557677 }
```

Lambda 1792MB

```
Count { all: 15408000, tried: 15408000, valid: 8160671, ac: 623972 }
```

## Lambda

base image: https://gallery.ecr.aws/lambda/provided

### image

169698630369.dkr.ecr.ap-northeast-1.amazonaws.com/ahc001

* 20210309-155930: f5e0eb7341e81c9c7697d823b387260756b629ac
* 20210309-210840: b9844151804abf57162aa0f60dc6c0f83126cbe2 「Rect の単純な水平移動は削除」

c71a3557ac86a9f74763e0ecd8dab1448be68a92

```
>>> d =  {
    'temp0': 0.10868564634648839, 
    'temp1': 0.00029342425784192465,
    'move_d_max': 61,
    'grow_d1_start': 661.4780032749206,
    'grow_d1_end': 7.211273402804876,
    'grow_d2_start': 1532.4395218778254,
    'grow_d2_end': 3.178973798285788,
    'rect_move_weight': 0.0691745158588457,
    'rect_grow_d1_weight': 0.21424863640912867,
    'rect_grow_d2_weight': 0.9440859754848975
}
>>> (1 - 0.012367993809999978) / 2
0.493816003095
```

### deploy

```
npm run build
npx cdk deploy --profile=ahc001
```

### Lambda を直接 invoke するテスト

```
aws lambda invoke \
    --function-name ahc001 \
    --payload $(base64 lambda-test-body.json) \
    --invocation-type Event /dev/null
aws sqs receive-message --queue-url https://sqs.ap-northeast-1.amazonaws.com/169698630369/ahc001-Queue4A7E3555-RODR3OR8LUGQ > a.json
aws sqs delete-message --receipt-handle $(jq -r '.Messages[0].ReceiptHandle' a.json)  --queue-url https://sqs.ap-northeast-1.amazonaws.com/169698630369/ahc001-Queue4A7E3555-RODR3OR8LUGQ
```
