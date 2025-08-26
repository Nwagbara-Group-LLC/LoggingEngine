{{/*
Expand the name of the chart.
*/}}
{{- define "ultra-logging-engine.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "ultra-logging-engine.fullname" -}}
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
{{- define "ultra-logging-engine.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "ultra-logging-engine.labels" -}}
helm.sh/chart: {{ include "ultra-logging-engine.chart" . }}
{{ include "ultra-logging-engine.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: trading-platform
{{- end }}

{{/*
Selector labels
*/}}
{{- define "ultra-logging-engine.selectorLabels" -}}
app.kubernetes.io/name: {{ include "ultra-logging-engine.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "ultra-logging-engine.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "ultra-logging-engine.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Ultra-logger labels
*/}}
{{- define "ultra-logger.labels" -}}
{{ include "ultra-logging-engine.labels" . }}
app.kubernetes.io/component: ultra-logger
{{- end }}

{{/*
Ultra-logger selector labels
*/}}
{{- define "ultra-logger.selectorLabels" -}}
{{ include "ultra-logging-engine.selectorLabels" . }}
app.kubernetes.io/component: ultra-logger
{{- end }}

{{/*
Log aggregator labels
*/}}
{{- define "log-aggregator.labels" -}}
{{ include "ultra-logging-engine.labels" . }}
app.kubernetes.io/component: log-aggregator
{{- end }}

{{/*
Log aggregator selector labels
*/}}
{{- define "log-aggregator.selectorLabels" -}}
{{ include "ultra-logging-engine.selectorLabels" . }}
app.kubernetes.io/component: log-aggregator
{{- end }}

{{/*
Metrics collector labels
*/}}
{{- define "metrics-collector.labels" -}}
{{ include "ultra-logging-engine.labels" . }}
app.kubernetes.io/component: metrics-collector
{{- end }}

{{/*
Metrics collector selector labels
*/}}
{{- define "metrics-collector.selectorLabels" -}}
{{ include "ultra-logging-engine.selectorLabels" . }}
app.kubernetes.io/component: metrics-collector
{{- end }}

{{/*
Trading-optimized resource configuration
*/}}
{{- define "trading.resources" -}}
{{- if .performance.enableHugepages }}
hugepages-2Mi: {{ .resources.requests.memory }}
{{- end }}
{{- end }}

{{/*
Trading-optimized node selector
*/}}
{{- define "trading.nodeSelector" -}}
{{- if .performance.numaTopology }}
node.kubernetes.io/numa-topology: "true"
{{- end }}
{{- if .performance.cpuAffinity }}
node.kubernetes.io/cpu-manager-policy: "static"
{{- end }}
{{- end }}

{{/*
Performance annotations
*/}}
{{- define "trading.annotations" -}}
{{- if .performance.enableHugepages }}
kubernetes.io/hugepages: "true"
{{- end }}
{{- if .performance.cpuAffinity }}
cpu-manager.alpha.kubernetes.io/cpuset: {{ .performance.isolateCpus | quote }}
{{- end }}
{{- end }}
