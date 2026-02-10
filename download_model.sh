#!/bin/bash
set -e

echo "Downloading Cortex Model (BERT-Base-NER)..."
mkdir -p model

# Config
if [ ! -f model/config.json ]; then
    curl -L -o model/config.json https://huggingface.co/dslim/bert-base-NER/resolve/main/config.json
    echo "Config downloaded."
fi

# Tokenizer (dslim/bert-base-NER lacks tokenizer.json, using base model)
if [ ! -f model/tokenizer.json ]; then
    curl -L -o model/tokenizer.json https://huggingface.co/bert-base-cased/resolve/main/tokenizer.json
    echo "Tokenizer downloaded (from bert-base-cased)."
fi

# Model Weights
if [ ! -f model/model.safetensors ]; then
    echo "Downloading weights (~400MB)..."
    curl -L -o model/model.safetensors https://huggingface.co/dslim/bert-base-NER/resolve/main/model.safetensors
    echo "Weights downloaded."
fi

echo "Cortex Model Ready."
