{{- define "push.name" -}}{{ .Release.Name }}-kura-push-gateway{{- end }}
{{- define "push.labels" -}}
app.kubernetes.io/name: kura-push-gateway
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
{{- define "push.runtimeLabels" -}}
{{ include "push.labels" . }}
app.kubernetes.io/component: runtime
{{- end }}
{{- define "push.migrationLabels" -}}
{{ include "push.labels" . }}
app.kubernetes.io/component: migration
{{- end }}
