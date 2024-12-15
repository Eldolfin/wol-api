<script setup lang="ts">
import "./main.css";
import MachineList from "./components/MachineList.vue";

import createClient from "openapi-fetch";
import { api_client, type ApiClient } from "./provides";

const baseUrl = import.meta.env.VITE_WOL_API_URL;

provide(api_client, createClient({ baseUrl }));
</script>
<template>
  <MachineList />
</template>
