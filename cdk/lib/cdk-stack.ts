import * as cdk from '@aws-cdk/core';
import * as lambda from "@aws-cdk/aws-lambda";
import * as ecr from "@aws-cdk/aws-ecr";
import { Role, ServicePrincipal, ManagedPolicy } from "@aws-cdk/aws-iam";

export class CdkStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const repository = ecr.Repository.fromRepositoryArn(this, 'ECR', 'arn:aws:ecr:ap-northeast-1:169698630369:repository/ahc001');
    const role = new Role(this, 'ExecRole', {
      roleName: 'ahc001-lambda-role',
      assumedBy: new ServicePrincipal('lambda.amazonaws.com'),
      managedPolicies: [
        ManagedPolicy.fromAwsManagedPolicyName('service-role/AWSLambdaBasicExecutionRole'),
      ],
    });

    const func = new lambda.Function(this, 'lambda', {
      functionName: 'ahc001',
      code: lambda.Code.fromEcrImage(repository, {
        tag: '20210309-155930',
      }),
      role,
      runtime: lambda.Runtime.FROM_IMAGE,
      handler: lambda.Handler.FROM_IMAGE,
      timeout: cdk.Duration.seconds(10),
    });

    new cdk.CfnOutput(this, 'LambdaArn', {
      value: func.functionArn,
    });
  }
}
