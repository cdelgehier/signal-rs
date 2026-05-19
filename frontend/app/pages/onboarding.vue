<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function onPaired() {
  router.replace('/')
}
</script>

<template>
  <div class="h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950">
    <QrCodeView @paired="onPaired" />
  </div>
</template>
