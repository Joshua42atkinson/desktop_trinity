#!/bin/bash
# Exit immediately if a command exits with a non-zero status.
set -e

# Build the Docker image
gcloud builds submit --tag gcr.io/$GOOGLE_CLOUD_PROJECT/daydream-mvp

# Deploy the image to Cloud Run
gcloud run deploy daydream-mvp \
  --image gcr.io/$GOOGLE_CLOUD_PROJECT/daydream-mvp \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars LEPTOS_SITE_ROOT=site
