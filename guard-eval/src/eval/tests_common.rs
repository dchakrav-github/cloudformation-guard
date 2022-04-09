use super::*;
use crate::Comparison;

#[derive(Debug)]
pub(super) struct NoOpReporter{}

impl<'value> EvalReporter<'value> for NoOpReporter {
    fn report_missing_value(&mut self,
                            _until: ValueType<'value>,
                            _data_file_name: &'value str,
                            _expr: &'value Expr)
        -> Result<(), std::io::Error> {
        Ok(())
    }

    fn report_mismatch_value_traversal(&mut self,
                                       _until: ValueType<'value>,
                                       _data_file_name: &'value str,
                                       _expr: &'value Expr)
        -> Result<(), std::io::Error> {
        Ok(())
    }

    fn report_evaluation(&mut self,
		 status: Status,
		 comparison: Comparison<'value>,
		 data_file: &'value str,
		 expr: &'value Expr) -> Result<(),
		 std::io::Error> {
        Ok(())
    }
}

pub(super) fn get_value() -> Result<Value, EvaluationError<'static>> {
    crate::value_internal::read_from(get_value_yaml())
}

fn get_value_yaml() -> &'static str {
    r###"
Resources:
  WebServerGroup:
    Type: AWS::AutoScaling::AutoScalingGroup
    CreationPolicy:
      ResourceSignal:
        Timeout: PT15M
        Count: '2'
    UpdatePolicy:
      AutoScalingRollingUpdate:
        MaxBatchSize: '1'
        MinInstancesInService: '1'
        PauseTime: PT15M
        WaitOnResourceSignals: 'true'
    Properties:
      AvailabilityZones: !GetAZs ''
      LaunchConfigurationName: !Ref 'LaunchConfig'
      MinSize: '2'
      MaxSize: '4'
      LoadBalancerNames: [!Ref 'ElasticLoadBalancer']
  LaunchConfig:
    Type: AWS::AutoScaling::LaunchConfiguration
    Metadata:
      AWS::CloudFormation::Init:
        configSets:
          full_install: [install_cfn, install_app, verify_instance_health]
        install_cfn:
          files:
            /etc/cfn/cfn-hup.conf:
              content: !Sub |
                [main]
                stack=${AWS::StackId}
                region=${AWS::Region}
              mode: '000400'
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Sub |
                [cfn-auto-reloader-hook]
                triggers=post.update
                path=Resources.LaunchConfig.Metadata.AWS::CloudFormation::Init
                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource LaunchConfig --configsets full_install --region ${AWS::Region}
                runas=root
          services:
            sysvinit:
              cfn-hup:
                enabled: 'true'
                ensureRunning: 'true'
                files: [/etc/cfn/cfn-hup.conf, /etc/cfn/hooks.d/cfn-auto-reloader.conf]
        install_app:
          packages:
            yum:
              httpd: []
          files:
            /var/www/html/index.html:
              content: !Join
                - ''
                - - '<h1>Congratulations, you have successfully launched the AWS CloudFormation sample.</h1>'
                  - '<p>Version: 1.0</p>'
              mode: '000644'
              owner: root
              group: root
          services:
            sysvinit:
              httpd:
                enabled: 'true'
                ensureRunning: 'true'
        verify_instance_health:
          commands:
            ELBHealthCheck:
              command: !Sub
                'until [ "$state" == "\"InService\"" ]; do state=$(aws --region ${AWS::Region} elb describe-instance-health
                 --load-balancer-name ${ElasticLoadBalancer}
                 --instances $(curl -s http://169.254.169.254/latest/meta-data/instance-id)
                 --query InstanceStates[0].State); sleep 10; done'
    Properties:
      KeyName: !Ref 'KeyName'
      ImageId: !FindInMap [AWSRegionArch2AMI, !Ref 'AWS::Region', !FindInMap [AWSInstanceType2Arch,
          !Ref 'InstanceType', Arch]]
      InstanceType: !Ref 'InstanceType'
      SecurityGroups: [!Ref 'InstanceSecurityGroup']
      IamInstanceProfile: !Ref 'WebServerInstanceProfile'
      UserData:
        Fn::Base64: !Sub |
          #!/bin/bash -xe
          yum install -y aws-cfn-bootstrap
          /opt/aws/bin/cfn-init -v --stack ${AWS::StackId} --resource LaunchConfig --configsets full_install --region ${AWS::Region}
          /opt/aws/bin/cfn-signal -e $? --stack ${AWS::StackId} --resource WebServerGroup --region ${AWS::Region}
  ElasticLoadBalancer:
    Type: AWS::ElasticLoadBalancing::LoadBalancer
    Properties:
      AvailabilityZones: !GetAZs ''
      CrossZone: 'true'
      Listeners:
      - LoadBalancerPort: '80'
        InstancePort: '80'
        Protocol: HTTP
      HealthCheck:
        Target: HTTP:80/
        HealthyThreshold: '3'
        UnhealthyThreshold: '5'
        Interval: '30'
        Timeout: '5'
  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access and HTTP access on the configured port
      SecurityGroupIngress:
      - IpProtocol: tcp
        FromPort: '22'
        ToPort: '22'
        CidrIp: !Ref 'SSHLocation'
      - IpProtocol: tcp
        FromPort: '80'
        ToPort: '80'
        CidrIp: 0.0.0.0/0
  WebServerInstanceProfile:
    Type: AWS::IAM::InstanceProfile
    Properties:
      Path: /
      Roles: [!Ref 'DescribeHealthRole']
  DescribeHealthRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Statement:
        - Effect: Allow
          Principal:
            Service: [ec2.amazonaws.com]
          Action: ['sts:AssumeRole']
      Path: /
      Policies:
      - PolicyName: describe-instance-health-policy
        PolicyDocument:
          Statement:
          - Effect: Allow
            Action: ['elasticloadbalancing:DescribeInstanceHealth']
            Resource: '*'
Outputs:
  URL:
    Description: URL of the website
    Value: !Join ['', ['http://', !GetAtt [ElasticLoadBalancer, DNSName]]]

    "###
}
