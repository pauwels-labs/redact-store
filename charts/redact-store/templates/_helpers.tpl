{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "labels" -}}
helm.sh/chart: {{ include "chart" . }}
{{ include "selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "selectorLabels" -}}
app.kubernetes.io/name: {{ include "name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "parameterizedHost" -}}
{{- if .Rule.host -}}
{{- if gt (len .Rule.host) 0 -}}
{{ .Rule.host }}
{{- end -}}
{{- else if .Rule.useEnvRootHost -}}
{{- printf "%s%s" (default "" .Rule.subdomain) (include "baseEnvUrl" .Global) -}}
{{- else if .Rule.useRootHost -}}
{{- printf "%s%s" (default "" .Rule.subdomain) .Global.Values.jxRequirements.ingress.domain -}}
{{- else -}}
{{- include "url" .Global  }}
{{- end -}}
{{- end -}}

{{/*
Create the name of the service account to use
*/}}
{{- define "serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

# TEMPORARY: Until jx provides a way to generically override Helm values dependent on the env an
#            app is being deployed to, we must infer the environment name from the namespace being
#            deployed to. This is fragile as the namespace name may not necessarily match the environment
#            name in the long run but it's what we have. The other option is to infer the value from
#            the ingress URL, i.e. .Values.jxRequirements.ingress.namespaceSubdomain.
#            Ideally, jx will provide a way to add arbitrary values to .Values.jxRequirements that
#            depend on the namespace being deployed to.
{{- define "environmentName" -}}
{{- $envFromNamespace := (default "default" (trimPrefix "jx-" .Release.Namespace)) -}}
{{- if eq $envFromNamespace "development" -}}
dev
{{- else -}}
{{ $envFromNamespace }}
{{- end -}}
{{- end -}}

{{- define "baseEnvUrl" -}}
{{- printf "%s%s" (trimPrefix "." .Values.jxRequirements.ingress.namespaceSubDomain) .Values.jxRequirements.ingress.domain -}}
{{- end -}}

{{- define "url" -}}
{{- $url := printf "%s.%s" (include "name" .) (include "baseEnvUrl" .) -}}
{{- default $url .Values.ingress.urlOverride -}}
{{- end -}}

{{- define "volumes" -}}
{{- if hasKey .Values "extraVolumes" }}
{{- $lenExtraVolumes := len .Values.extraVolumes }}
{{- if gt $lenExtraVolumes 0 }}
{{ toYaml .Values.extraVolumes }}
{{- end }}
{{- end }}
{{- end }}

{{- define "volumeMounts" -}}
{{- if hasKey .Values.extraVolumeMounts .ContainerName }}
{{- $extraVolumeMounts := index .Values.extraVolumeMounts .ContainerName }}
{{- $lenExtraVolumeMounts := len $extraVolumeMounts }}
{{- if gt $lenExtraVolumeMounts 0 }}
volumeMounts:
{{ toYaml $extraVolumeMounts }}
{{- end }}
{{- else if hasKey .Values.extraVolumeMounts "default" }}
{{- $extraVolumeMounts := index .Values.extraVolumeMounts "default" }}
{{- $lenExtraVolumeMounts := len $extraVolumeMounts }}
{{- if gt $lenExtraVolumeMounts 0 }}
volumeMounts:
{{ toYaml $extraVolumeMounts }}
{{- end }}
{{- end }}
{{- end }}

{{- define "envVars" -}}
{{- if hasKey .Values.env .ContainerName -}}
{{- $extraEnvs := index .Values.env .ContainerName -}}
{{- $lenExtraEnvs := len $extraEnvs -}}
{{- if gt $lenExtraEnvs 0 -}}
env:
{{ toYaml $extraEnvs }}
{{- end -}}
{{- else if hasKey .Values.env "default" -}}
{{- $extraEnvs := index .Values.env "default" -}}
{{- $lenExtraEnvs := len $extraEnvs -}}
{{- if gt $lenExtraEnvs 0 -}}
env:
{{ toYaml $extraEnvs }}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "resources" -}}
{{- if hasKey .Values.resources .ContainerName }}
resources:
{{ toYaml (index .Values.resources .ContainerName) }}
{{- else if hasKey .Values.resources "default" }}
resources:
{{ toYaml .Values.resources.default | trim | indent 2 }}
{{- else }}
resources:
  limits:
    cpu: 1000m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi
{{- end }}
{{- end }}

{{- define "tlsSecretName" -}}
{{- $secretName := printf "tls-%s" (include "name" .) -}}
{{- default $secretName .Values.jxRequirements.ingress.tls.secretName -}}
{{- end -}}
