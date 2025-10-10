<template>
  <div v-html="LICENSE_INFO_SHORT_HTML"></div>
</template>

<script setup lang="ts">
import { LICENSE_INFO_SHORT_HTML } from '@/sp2any.bindings'
</script>
<style>
.license-short a {
  color: var(--color-secondary);
}
</style>
