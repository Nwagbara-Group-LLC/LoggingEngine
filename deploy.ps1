# Ultra-Low Latency Logging Engine Deployment Script
# Optimized for high-frequency trading environments

param(
    [Parameter(Mandatory=$false)]
    [string]$Namespace = "ultra-logging",
    
    [Parameter(Mandatory=$false)]
    [string]$HelmReleaseName = "ultra-logging-engine",
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("dev", "staging", "production")]
    [string]$Environment = "dev",
    
    [Parameter(Mandatory=$false)]
    [switch]$BuildImages = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$EnablePerformanceOptimizations = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$EnableRedis = $true,
    
    [Parameter(Mandatory=$false)]
    [switch]$EnableKafka = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$EnablePrometheus = $true,
    
    [Parameter(Mandatory=$false)]
    [string]$DockerRegistry = "docker.io/trading-platform"
)

Write-Host "=== Ultra-Low Latency Logging Engine Deployment ===" -ForegroundColor Cyan
Write-Host "Environment: $Environment" -ForegroundColor Yellow
Write-Host "Namespace: $Namespace" -ForegroundColor Yellow
Write-Host "Helm Release: $HelmReleaseName" -ForegroundColor Yellow

# Check prerequisites
Write-Host "`n=== Checking Prerequisites ===" -ForegroundColor Cyan

# Check if kubectl is available
try {
    $kubectlVersion = kubectl version --client --short
    Write-Host "✅ kubectl: $kubectlVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ kubectl not found. Please install kubectl." -ForegroundColor Red
    exit 1
}

# Check if helm is available
try {
    $helmVersion = helm version --short
    Write-Host "✅ helm: $helmVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ helm not found. Please install helm." -ForegroundColor Red
    exit 1
}

# Check if docker is available (if building images)
if ($BuildImages) {
    try {
        $dockerVersion = docker version --format "{{.Client.Version}}"
        Write-Host "✅ docker: $dockerVersion" -ForegroundColor Green
    } catch {
        Write-Host "❌ docker not found. Please install docker." -ForegroundColor Red
        exit 1
    }
}

# Create namespace if it doesn't exist
Write-Host "`n=== Setting up Namespace ===" -ForegroundColor Cyan
kubectl create namespace $Namespace --dry-run=client -o yaml | kubectl apply -f -
kubectl label namespace $Namespace trading.platform/environment=$Environment --overwrite
Write-Host "✅ Namespace '$Namespace' ready" -ForegroundColor Green

# Build Docker images if requested
if ($BuildImages) {
    Write-Host "`n=== Building Docker Images ===" -ForegroundColor Cyan
    
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $rootDir = Split-Path -Parent $scriptDir
    
    # Build ultra-logger image
    Write-Host "Building ultra-logger image..." -ForegroundColor Yellow
    Set-Location "$rootDir"
    docker build -t "$DockerRegistry/ultra-logger:latest" -f Dockerfile.ultra-logger .
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ ultra-logger image built successfully" -ForegroundColor Green
        docker push "$DockerRegistry/ultra-logger:latest"
    } else {
        Write-Host "❌ Failed to build ultra-logger image" -ForegroundColor Red
        exit 1
    }
    
    # Build log-aggregator image
    Write-Host "Building log-aggregator image..." -ForegroundColor Yellow
    docker build -t "$DockerRegistry/log-aggregator:latest" -f Dockerfile.log-aggregator .
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ log-aggregator image built successfully" -ForegroundColor Green
        docker push "$DockerRegistry/log-aggregator:latest"
    } else {
        Write-Host "❌ Failed to build log-aggregator image" -ForegroundColor Red
        exit 1
    }
    
    # Build metrics-collector image
    Write-Host "Building metrics-collector image..." -ForegroundColor Yellow
    docker build -t "$DockerRegistry/metrics-collector:latest" -f Dockerfile.metrics-collector .
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ metrics-collector image built successfully" -ForegroundColor Green
        docker push "$DockerRegistry/metrics-collector:latest"
    } else {
        Write-Host "❌ Failed to build metrics-collector image" -ForegroundColor Red
        exit 1
    }
    
    Set-Location $scriptDir
}

# Add required Helm repositories
Write-Host "`n=== Adding Helm Repositories ===" -ForegroundColor Cyan
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
Write-Host "✅ Helm repositories updated" -ForegroundColor Green

# Prepare values file based on environment
Write-Host "`n=== Preparing Configuration ===" -ForegroundColor Cyan

$valuesOverrides = @()

# Environment-specific configurations
switch ($Environment) {
    "dev" {
        $valuesOverrides += "--set ultraLogger.replicaCount=1"
        $valuesOverrides += "--set logAggregator.replicaCount=1"
        $valuesOverrides += "--set metricsCollector.replicaCount=1"
        $valuesOverrides += "--set ultraLogger.resources.requests.cpu=100m"
        $valuesOverrides += "--set ultraLogger.resources.requests.memory=256Mi"
        $valuesOverrides += "--set ultraLogger.persistence.size=10Gi"
    }
    "staging" {
        $valuesOverrides += "--set ultraLogger.replicaCount=2"
        $valuesOverrides += "--set logAggregator.replicaCount=2"
        $valuesOverrides += "--set metricsCollector.replicaCount=1"
        $valuesOverrides += "--set ultraLogger.resources.requests.cpu=500m"
        $valuesOverrides += "--set ultraLogger.resources.requests.memory=1Gi"
        $valuesOverrides += "--set ultraLogger.persistence.size=50Gi"
    }
    "production" {
        $valuesOverrides += "--set ultraLogger.replicaCount=3"
        $valuesOverrides += "--set logAggregator.replicaCount=3"
        $valuesOverrides += "--set metricsCollector.replicaCount=2"
        $valuesOverrides += "--set ultraLogger.resources.requests.cpu=1000m"
        $valuesOverrides += "--set ultraLogger.resources.requests.memory=2Gi"
        $valuesOverrides += "--set ultraLogger.persistence.size=100Gi"
        $valuesOverrides += "--set autoscaling.enabled=true"
        $valuesOverrides += "--set podDisruptionBudget.enabled=true"
    }
}

# Performance optimizations
if ($EnablePerformanceOptimizations) {
    $valuesOverrides += "--set ultraLogger.performance.enableHugepages=true"
    $valuesOverrides += "--set ultraLogger.performance.cpuAffinity=true"
    $valuesOverrides += "--set ultraLogger.performance.numaTopology=true"
    $valuesOverrides += "--set ultraLogger.performance.isolateCpus=2-5"
    Write-Host "✅ Performance optimizations enabled" -ForegroundColor Green
}

# Storage backend configurations
if ($EnableRedis) {
    $valuesOverrides += "--set redis.enabled=true"
    Write-Host "✅ Redis storage backend enabled" -ForegroundColor Green
} else {
    $valuesOverrides += "--set redis.enabled=false"
}

if ($EnableKafka) {
    $valuesOverrides += "--set kafka.enabled=true"
    Write-Host "✅ Kafka streaming backend enabled" -ForegroundColor Green
} else {
    $valuesOverrides += "--set kafka.enabled=false"
}

if ($EnablePrometheus) {
    $valuesOverrides += "--set prometheus.enabled=true"
    Write-Host "✅ Prometheus monitoring enabled" -ForegroundColor Green
} else {
    $valuesOverrides += "--set prometheus.enabled=false"
}

# Docker registry configuration
$valuesOverrides += "--set ultraLogger.image.registry=$($DockerRegistry.Split('/')[0])"
$valuesOverrides += "--set ultraLogger.image.repository=$($DockerRegistry.Split('/')[1])/ultra-logger"
$valuesOverrides += "--set logAggregator.image.registry=$($DockerRegistry.Split('/')[0])"
$valuesOverrides += "--set logAggregator.image.repository=$($DockerRegistry.Split('/')[1])/log-aggregator"
$valuesOverrides += "--set metricsCollector.image.registry=$($DockerRegistry.Split('/')[0])"
$valuesOverrides += "--set metricsCollector.image.repository=$($DockerRegistry.Split('/')[1])/metrics-collector"

# Deploy using Helm
Write-Host "`n=== Deploying Ultra-Low Latency Logging Engine ===" -ForegroundColor Cyan

$helmCommand = @(
    "helm", "upgrade", "--install", $HelmReleaseName,
    "./helm-charts",
    "--namespace", $Namespace,
    "--create-namespace"
) + $valuesOverrides + @(
    "--wait", "--timeout=10m"
)

Write-Host "Executing: $($helmCommand -join ' ')" -ForegroundColor Gray
& $helmCommand[0] $helmCommand[1..($helmCommand.Length-1)]

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Deployment completed successfully" -ForegroundColor Green
} else {
    Write-Host "❌ Deployment failed" -ForegroundColor Red
    exit 1
}

# Verify deployment
Write-Host "`n=== Verifying Deployment ===" -ForegroundColor Cyan

# Check pod status
Write-Host "Checking pod status..." -ForegroundColor Yellow
kubectl get pods -n $Namespace -l "app.kubernetes.io/instance=$HelmReleaseName"

# Check service status
Write-Host "`nChecking service status..." -ForegroundColor Yellow
kubectl get services -n $Namespace -l "app.kubernetes.io/instance=$HelmReleaseName"

# Check persistent volume claims
Write-Host "`nChecking storage..." -ForegroundColor Yellow
kubectl get pvc -n $Namespace

# Wait for pods to be ready
Write-Host "`nWaiting for pods to be ready..." -ForegroundColor Yellow
kubectl wait --for=condition=ready pod -l "app.kubernetes.io/instance=$HelmReleaseName" -n $Namespace --timeout=300s

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ All pods are ready" -ForegroundColor Green
} else {
    Write-Host "⚠️  Some pods may still be starting" -ForegroundColor Yellow
}

# Display access information
Write-Host "`n=== Access Information ===" -ForegroundColor Cyan

# Ultra-logger service
$ultraLoggerService = kubectl get service "$HelmReleaseName-ultra-logger" -n $Namespace -o jsonpath='{.spec.clusterIP}:{.spec.ports[0].port}'
Write-Host "Ultra-Logger Service: http://$ultraLoggerService" -ForegroundColor Green

# Metrics collector service
if ($EnablePrometheus) {
    $metricsService = kubectl get service "$HelmReleaseName-metrics-collector" -n $Namespace -o jsonpath='{.spec.clusterIP}:{.spec.ports[1].port}'
    Write-Host "Metrics Endpoint: http://$metricsService/metrics" -ForegroundColor Green
}

# Log aggregator service
$aggregatorService = kubectl get service "$HelmReleaseName-log-aggregator" -n $Namespace -o jsonpath='{.spec.clusterIP}:{.spec.ports[0].port}'
Write-Host "Log Aggregator: tcp://$aggregatorService" -ForegroundColor Green

# Performance testing commands
Write-Host "`n=== Performance Testing ===" -ForegroundColor Cyan
Write-Host "To test ultra-low latency logging performance:" -ForegroundColor Yellow
Write-Host "kubectl exec -it deployment/$HelmReleaseName-ultra-logger -n $Namespace -- /bin/bash" -ForegroundColor Gray
Write-Host "# Inside the container:" -ForegroundColor Gray
Write-Host "# /usr/local/bin/ultra-logger-benchmark --test-duration=60s --target-latency=1us" -ForegroundColor Gray

Write-Host "`nTo monitor real-time metrics:" -ForegroundColor Yellow
Write-Host "kubectl port-forward service/$HelmReleaseName-metrics-collector -n $Namespace 9092:9092" -ForegroundColor Gray
Write-Host "# Then visit: http://localhost:9092/metrics" -ForegroundColor Gray

Write-Host "`n=== Deployment Summary ===" -ForegroundColor Cyan
Write-Host "Environment: $Environment" -ForegroundColor White
Write-Host "Namespace: $Namespace" -ForegroundColor White
Write-Host "Helm Release: $HelmReleaseName" -ForegroundColor White
Write-Host "Components Deployed:" -ForegroundColor White
Write-Host "  - Ultra-Logger (ultra-low latency logging)" -ForegroundColor White
Write-Host "  - Log Aggregator (log routing and collection)" -ForegroundColor White
Write-Host "  - Metrics Collector (performance monitoring)" -ForegroundColor White
if ($EnableRedis) { Write-Host "  - Redis (storage backend)" -ForegroundColor White }
if ($EnableKafka) { Write-Host "  - Kafka (streaming backend)" -ForegroundColor White }
if ($EnablePrometheus) { Write-Host "  - Prometheus (metrics storage)" -ForegroundColor White }

Write-Host "`n🚀 Ultra-Low Latency Logging Engine deployed successfully!" -ForegroundColor Green
Write-Host "Ready for high-frequency trading workloads with microsecond-precision logging." -ForegroundColor Green
