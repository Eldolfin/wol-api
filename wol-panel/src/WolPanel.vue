<script setup lang="ts">
import "./main.css";
import MachineList from "./components/MachineList.vue";

import createClient from "openapi-fetch";
import { api_client, baseUrl } from "./provides";

provide(api_client, createClient({ baseUrl: baseUrl.origin }));
</script>
<template>
  <MachineList />
</template>
