#!/bin/bash
# Verostark Manual Deployment Script
# Usage: ./deploy_manual.sh

echo "🚀 Verostark Deployment: Manual Push to Scaleway"

# 1. Configuration (Users must set these or provide via ENV)
REGION="fr-par"
REGISTRY_NAMESPACE="verostark" # Default guess, change if needed
IMAGE_NAME="verostark:cortex"
REMOTE_TAG="rg.${REGION}.scw.cloud/${REGISTRY_NAMESPACE}/${IMAGE_NAME}"

echo "Checking Docker login..."
# Validate login - simple check if config exists or prompt
if ! docker login rg.${REGION}.scw.cloud > /dev/null 2>&1; then
    echo "⚠️  Not logged in to Scaleway Registry."
    echo "Please run: docker login rg.${REGION}.scw.cloud -u nologin -p \$SCW_SECRET_KEY"
    read -p "Press Enter when logged in..."
fi

echo "📦 Tagging image..."
docker tag ${IMAGE_NAME} ${REMOTE_TAG}

echo "⬆️  Pushing image to ${REMOTE_TAG}..."
docker push ${REMOTE_TAG}

echo "✅ Deployment Push Complete!"
echo "Next Step: Go to Scaleway Console -> Serverless Containers -> Redeploy"
