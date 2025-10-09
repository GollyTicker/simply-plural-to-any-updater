<template>
  <div v-html="LISENCE_WITH_VERSION"></div>
</template>

<script setup lang="ts">
import { LICENSE_INFO_SHORT_HTML, SP2ANY_VERSION } from '@/sp2any.bindings'
const LISENCE_WITH_VERSION = SP2ANY_VERSION + ' | ' + LICENSE_INFO_SHORT_HTML
</script>
<style>
.license-short a {
  color: var(--color-secondary);
}
</style>
