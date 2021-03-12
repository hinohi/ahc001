import * as cdk from '@aws-cdk/core';
import * as lambda from '@aws-cdk/aws-lambda';
import * as ecr from '@aws-cdk/aws-ecr';
import * as sqs from '@aws-cdk/aws-sqs';
import {Role, ServicePrincipal, ManagedPolicy} from '@aws-cdk/aws-iam';
import {SqsDestination} from '@aws-cdk/aws-lambda-destinations';

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
    const queue = new sqs.Queue(this, 'Queue', {
      retentionPeriod: cdk.Duration.hours(1),
      visibilityTimeout: cdk.Duration.seconds(60),
    });

    const func = new lambda.Function(this, 'lambda', {
      functionName: 'ahc001',
      code: lambda.Code.fromEcrImage(repository, {
        tag: '20210312-131142',
      }),
      role,
      runtime: lambda.Runtime.FROM_IMAGE,
      handler: lambda.Handler.FROM_IMAGE,
      timeout: cdk.Duration.seconds(10),
      onSuccess: new SqsDestination(queue),
      maxEventAge: cdk.Duration.minutes(10),
      retryAttempts: 0,
    });

    new cdk.CfnOutput(this, 'LambdaArn', {
      value: func.functionArn,
    });
    new cdk.CfnOutput(this, 'OutputQueueUrl', {
      value: queue.queueUrl,
    });
  }
}
