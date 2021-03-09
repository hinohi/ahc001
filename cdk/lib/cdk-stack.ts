import * as cdk from '@aws-cdk/core';
import * as lambda from '@aws-cdk/aws-lambda';
import * as ecr from '@aws-cdk/aws-ecr';
import * as sqs from '@aws-cdk/aws-sqs';
import {Role, ServicePrincipal, ManagedPolicy} from '@aws-cdk/aws-iam';
import {SqsDestination} from '@aws-cdk/aws-lambda-destinations';
import {SqsEventSource} from '@aws-cdk/aws-lambda-event-sources';

export class CdkStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const repository = ecr.Repository.fromRepositoryArn(this, 'ECR', 'arn:aws:ecr:ap-northeast-1:169698630369:repository/ahc001');
    const role = new Role(this, 'ExecRole', {
      roleName: 'ahc001-lambda-role',
      assumedBy: new ServicePrincipal('lambda.amazonaws.com'),
      managedPolicies: [
        ManagedPolicy.fromAwsManagedPolicyName('service-role/AWSLambdaBasicExecutionRole'),
        ManagedPolicy.fromAwsManagedPolicyName('service-role/AWSLambdaSQSQueueExecutionRole'),
      ],
    });
    const inputQueue = new sqs.Queue(this, 'InputQueue', {
      retentionPeriod: cdk.Duration.hours(1),
      visibilityTimeout: cdk.Duration.seconds(60),
    });
    const outputQueue = new sqs.Queue(this, 'OutputQueue', {
      retentionPeriod: cdk.Duration.hours(1),
      visibilityTimeout: cdk.Duration.seconds(60),
    });

    const func = new lambda.Function(this, 'lambda', {
      functionName: 'ahc001',
      code: lambda.Code.fromEcrImage(repository, {
        tag: '20210309-191804',
      }),
      role,
      runtime: lambda.Runtime.FROM_IMAGE,
      handler: lambda.Handler.FROM_IMAGE,
      timeout: cdk.Duration.seconds(10),
      onSuccess: new SqsDestination(outputQueue),
    });
    func.addEventSource(new SqsEventSource(inputQueue, {batchSize: 1}));

    new cdk.CfnOutput(this, 'LambdaArn', {
      value: func.functionArn,
    });
    new cdk.CfnOutput(this, 'InputQueueUrl', {
      value: inputQueue.queueUrl,
    });
    new cdk.CfnOutput(this, 'OutputQueueUrl', {
      value: outputQueue.queueUrl,
    });
  }
}
