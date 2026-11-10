import * as kubernetes from '@pulumi/kubernetes';
import * as pulumi from '@pulumi/pulumi';
import * as docker from '@pulumi/docker';
import * as cp from 'node:child_process';

const sha = new Promise(async (resolve) => {
  const out = cp.execSync('git rev-parse --short HEAD');
  resolve(Buffer.from(out).toString().trim());
});

const cluster_reference = new pulumi.StackReference('cluster', {
  name: 'julienvincent/daedalus-cluster-master/root',
});

const k8s = new pulumi.StackReference('kubernetes', {
  name: 'julienvincent/daedalus-kubernetes/root',
});

const provider = new kubernetes.Provider('root-provider', {
  kubeconfig: cluster_reference.requireOutput('kubeconfig_yaml'),
  clusterIdentifier: 'daedalus',
});

const issuers = k8s.requireOutput('issuers') as pulumi.Output<
  Record<string, string>
>;
const gateways = k8s.requireOutput('gateways') as pulumi.Output<{
  internal: { name: string; namespace: string };
}>;

const api_ref = new pulumi.StackReference('rebound-api', {
  name: 'julienvincent/rebound-api/root',
});

const image = new docker.Image('api', {
  imageName: pulumi.interpolate`registry.julienvincent.io/rebound-ui:${sha}`,
  build: {
    context: '../../',
    dockerfile: '../Dockerfile',
    platform: 'linux/arm64',
  },
});

const options = {
  provider,
};

const namespace = api_ref.requireOutput('namespace');

const labels = {
  'app.kubernetes.io/name': 'rebound-ui',
};

const config = new kubernetes.core.v1.ConfigMap(
  'ui-config',
  {
    metadata: {
      namespace,
    },
    immutable: true,
    data: {
      'config.json': JSON.stringify({
        api_endpoint: 'https://api.rebound.julienvincent.io',
      }),
    },
  },
  {
    provider,
  },
);

new kubernetes.apps.v1.Deployment(
  'ui',
  {
    metadata: {
      namespace,
      labels,
    },
    spec: {
      selector: {
        matchLabels: labels,
      },
      replicas: 2,
      template: {
        metadata: {
          labels,
        },
        spec: {
          nodeSelector: {
            'daedalus/network': 'home',
          },
          volumes: [
            {
              name: 'config',
              configMap: {
                name: config.metadata.name,
                items: [
                  {
                    key: 'config.json',
                    path: 'config.json',
                  },
                ],
              },
            },
          ],
          containers: [
            {
              name: 'default',
              image: image.imageName,
              resources: {
                requests: {
                  memory: '50Mi',
                  cpu: '10m',
                },
                limits: {
                  memory: '50Mi',
                },
              },
              ports: [
                {
                  containerPort: 80,
                  name: 'http',
                },
              ],
              volumeMounts: [
                {
                  name: 'config',
                  mountPath: '/app/static/config.json',
                  subPath: 'config.json',
                },
              ],

              startupProbe: {
                httpGet: {
                  path: '/',
                  port: 80,
                },
                periodSeconds: 1,
                failureThreshold: 500,
                successThreshold: 1,
                timeoutSeconds: 1,
              },
              livenessProbe: {
                httpGet: {
                  path: '/',
                  port: 80,
                },
                periodSeconds: 10,
                failureThreshold: 3,
                successThreshold: 1,
                timeoutSeconds: 1,
              },
            },
          ],
        },
      },
    },
  },
  options,
);
const service = new kubernetes.core.v1.Service(
  'ui',
  {
    metadata: {
      namespace,
      labels,
      annotations: {
        'pulumi.com/skipAwait': 'true',
      },
    },
    spec: {
      selector: labels,
      ports: [
        {
          port: 80,
          name: 'http',
          targetPort: 'http',
        },
      ],
    },
  },
  options,
);

const domain = 'rebound.julienvincent.io';
const cert_secret_name = 'rebound-ui-tls-tls';
const certificate = new kubernetes.apiextensions.CustomResource(
  'rebound-ui-tls',
  {
    apiVersion: 'cert-manager.io/v1',
    kind: 'Certificate',
    metadata: { namespace, labels: { solver: 'dns01' } },
    spec: {
      secretName: cert_secret_name,
      dnsNames: [domain],
      issuerRef: { kind: 'ClusterIssuer', name: issuers.letsencrypt },
    },
  },
  options,
);

const listener_set = new kubernetes.apiextensions.CustomResource(
  'rebound-ui',
  {
    apiVersion: 'gateway.networking.k8s.io/v1',
    kind: 'ListenerSet',
    metadata: { namespace },
    spec: {
      parentRef: {
        group: 'gateway.networking.k8s.io',
        kind: 'Gateway',
        name: gateways.internal.name,
        namespace: gateways.internal.namespace,
      },
      listeners: [
        {
          name: 'https-0',
          hostname: domain,
          port: 443,
          protocol: 'HTTPS',
          tls: {
            mode: 'Terminate',
            certificateRefs: [
              { group: '', kind: 'Secret', name: cert_secret_name },
            ],
          },
        },
      ],
    },
  },
  { ...options, dependsOn: certificate },
);

new kubernetes.apiextensions.CustomResource(
  'rebound-ui-https',
  {
    apiVersion: 'gateway.networking.k8s.io/v1',
    kind: 'HTTPRoute',
    metadata: { namespace },
    spec: {
      parentRefs: [
        {
          group: 'gateway.networking.k8s.io',
          kind: 'ListenerSet',
          name: listener_set.metadata.name,
          sectionName: 'https-0',
        },
      ],
      hostnames: [domain],
      rules: [
        {
          timeouts: { request: '0s', backendRequest: '0s' },
          backendRefs: [{ name: service.metadata.name, port: 80 }],
        },
      ],
    },
  },
  options,
);
