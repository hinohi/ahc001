# ahc001

## AtCoder 本番のめも

### ライブラリ

https://github.com/rust-lang-ja/atcoder-rust-resources/wiki/2020-Update

### 実行速度

手元

```console
$ /usr/bin/time ./target/release/local < tools/in/0000.txt > /dev/null
        0.45 real         0.45 user         0.00 sys
$ /usr/bin/time ./target/release/local < tools/in/0001.txt > /dev/null
        0.48 real         0.48 user         0.00 sys
$ /usr/bin/time ./target/release/local < tools/in/0002.txt > /dev/null
        0.50 real         0.49 user         0.00 sys
$ /usr/bin/time ./target/release/local < tools/in/0003.txt > /dev/null
        0.49 real         0.49 user         0.00 sys
```

提出

```text
提出日時	問題	ユーザ	言語	得点	コード長	結果	実行時間	メモリ	
2021-03-09 13:16:08	A - AtCoder Ad	daiju	Rust (1.42.0)	49057888001	17308 Byte		860 ms	3228 KB	詳細
```

倍とは言わないが、1.7倍くらいは遅い。

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
