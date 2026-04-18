import * as kubernetes from "@pulumi/kubernetes";
import * as postgres from "@pulumi/postgresql";
import * as tailscale from "@pulumi/tailscale";
import * as random from "@pulumi/random";
import * as pulumi from "@pulumi/pulumi";
import * as docker from "@pulumi/docker";
import * as cp from "node:child_process";

const config = new pulumi.Config();

const sha = new Promise(async (resolve) => {
  const out = cp.execSync("git rev-parse --short HEAD");
  resolve(Buffer.from(out).toString().trim());
});

const cluster_reference = new pulumi.StackReference("cluster", {
  name: "julienvincent/daedalus-cluster/root",
});

const k8s = new pulumi.StackReference("kubernetes", {
  name: "julienvincent/daedalus-kubernetes/root",
});

const postgres_stack = new pulumi.StackReference("postgres", {
  name: "julienvincent/daedalus-postgres/root",
});

const provider = new kubernetes.Provider("root-provider", {
  kubeconfig: cluster_reference.requireOutput("kubeconfig_yaml"),
});

const issuers = k8s.requireOutput("issuers") as pulumi.Output<
  Record<string, string>
>;

const image = new docker.Image("api", {
  imageName: pulumi.interpolate`registry.julienvincent.io/rebound-api:${sha}`,
  build: {
    context: "../../",
    dockerfile: "../Dockerfile",
    platform: "linux/arm64",
  },
});

const options = {
  provider,
};

export const namespace = new kubernetes.core.v1.Namespace(
  "rebound",
  {},
  {
    provider,
  },
).metadata.name;

const labels = {
  "app.kubernetes.io/name": "rebound-api",
};

const ts_oauth_credentials = new tailscale.OauthClient("oauth-credentials", {
  scopes: [
    "devices:core:read",
    "devices:posture_attributes:read",
    "policy_file:read",
    "auth_keys",
  ],
  description: "API access for rebound",
  tags: ["tag:exit-node"],
});

const postgres_connection_info = postgres_stack.requireOutput(
  "connection_info",
) as pulumi.Output<{
  external_host: string;
  host: string;
  port: number;
  database: string;
  root_username: string;
  root_password: string;
}>;

const postgres_provider = new postgres.Provider("postgres", {
  host: postgres_connection_info.external_host,
  port: postgres_connection_info.port,
  database: postgres_connection_info.database,
  username: postgres_connection_info.root_username,
  password: postgres_connection_info.root_password,
  sslmode: "disable",
});

const password = new random.RandomPassword("postgres-password", {
  length: 12,
  special: false,
}).result;
const user = new postgres.Role(
  "rebound",
  {
    login: true,
    password,
  },
  {
    provider: postgres_provider,
  },
);

const database = new postgres.Database(
  "rebound",
  {},
  {
    provider: postgres_provider,
  },
);

new postgres.Grant(
  `database-grant`,
  {
    database: database.name,
    role: user.name,
    objectType: "database",
    privileges: ["ALL"],
  },
  {
    provider: postgres_provider,
  },
);

const api_config = new kubernetes.core.v1.Secret(
  "config",
  {
    metadata: {
      namespace,
    },
    immutable: true,
    stringData: {
      "config.json": pulumi
        .output({
          postgres: {
            host: postgres_connection_info.host,
            port: postgres_connection_info.port,
            database: database.name,
            username: user.name,
            password: password,
          },

          fly: config.requireObject("fly"),

          tailscale: {
            client_id: ts_oauth_credentials.id,
            client_secret: ts_oauth_credentials.key,
            tailnet: config.require("tailnet"),
            exit_node_tag: "exit-node",
          },

          port: 80,
          watchdog_interval_secs: 60,
        })
        .apply(JSON.stringify),
    },
  },
  {
    provider,
  },
);

const volumes = [
  {
    name: "config",
    secret: {
      secretName: api_config.metadata.name,
    },
  },
];

const volumeMounts = [
  {
    name: "config",
    mountPath: "/config",
  },
];

const deployment = new kubernetes.apps.v1.Deployment(
  "api",
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
            "daedalus/network": "home",
          },
          volumes,
          containers: [
            {
              name: "default",
              image: image.imageName,
              args: ["serve", "--config", "/config/config.json"],
              resources: {
                requests: {
                  memory: "128Mi",
                  cpu: "10m",
                },
                limits: {
                  memory: "128Mi",
                },
              },
              ports: [
                {
                  containerPort: 80,
                  name: "http",
                },
              ],
              volumeMounts,

              startupProbe: {
                httpGet: {
                  path: "/livez",
                  port: 80,
                },
                periodSeconds: 1,
                failureThreshold: 500,
                successThreshold: 1,
                timeoutSeconds: 1,
              },
              livenessProbe: {
                httpGet: {
                  path: "/livez",
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

new kubernetes.batch.v1.Job(
  "migrate",
  {
    metadata: {
      namespace,
      annotations: {
        "pulumi.com/replaceUnready": "true",
      },
    },
    spec: {
      activeDeadlineSeconds: 600,
      ttlSecondsAfterFinished: 600,

      backoffLimit: 0,
      parallelism: 1,
      completions: 1,

      template: {
        metadata: {},
        spec: {
          restartPolicy: "Never",
          volumes: volumes,
          containers: [
            {
              name: "migrate",
              image: image.imageName,
              args: ["migrate", "--config", "/config/config.json"],
              volumeMounts,
              resources: {
                requests: {
                  memory: "50Mi",
                  cpu: "10m",
                },
                limits: {
                  memory: "50Mi",
                },
              },
            },
          ],
        },
      },
    },
  },
  {
    provider,
    dependsOn: [deployment],
  },
);

const service = new kubernetes.core.v1.Service(
  "ingress",
  {
    metadata: {
      namespace,
      labels,
      annotations: {
        "pulumi.com/skipAwait": "true",
      },
    },
    spec: {
      selector: labels,
      ports: [
        {
          port: 80,
          name: "http",
          targetPort: "http",
        },
      ],
    },
  },
  options,
);

const domain = "api.rebound.julienvincent.io";
const cert_secret_name = `api-rebound-julienvincent-io-tls`;
new kubernetes.apiextensions.CustomResource(
  "api-rebound-julienvincent-io",
  {
    apiVersion: "cert-manager.io/v1",
    kind: "Certificate",
    metadata: {
      namespace,
      labels: {
        solver: "dns01",
      },
    },
    spec: {
      secretName: cert_secret_name,
      dnsNames: [domain],
      issuerRef: {
        kind: "ClusterIssuer",
        name: issuers.letsencrypt,
      },
    },
  },
  options,
);

new kubernetes.networking.v1.Ingress(
  "api-rebound-julienvincent-io",
  {
    metadata: {
      namespace,
    },
    spec: {
      ingressClassName: "nginx-internal",
      tls: [
        {
          hosts: [domain],
          secretName: cert_secret_name,
        },
      ],
      rules: [
        {
          host: domain,
          http: {
            paths: [
              {
                pathType: "Prefix",
                path: "/",
                backend: {
                  service: {
                    name: service.metadata.name,
                    port: {
                      name: "http",
                    },
                  },
                },
              },
            ],
          },
        },
      ],
    },
  },
  options,
);
