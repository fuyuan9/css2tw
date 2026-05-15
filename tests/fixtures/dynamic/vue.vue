<template>
  <div class="static-class">
    <!-- Vue :class bindings (object syntax) -->
    <div :class="{ active: isActive, 'text-danger': hasError }">
      Object syntax
    </div>

    <!-- Vue :class bindings (array syntax) -->
    <div :class="[activeClass, errorClass]">
      Array syntax
    </div>

    <!-- Combined static and dynamic -->
    <div class="base" :class="{ 'is-loading': loading }">
      Combined
    </div>

    <!-- Computed class refs -->
    <div :class="computedClasses">
      Computed
    </div>

    <!-- Template expressions -->
    <div :class="isActive ? 'bg-blue' : 'bg-gray'">
      Ternary expression
    </div>

    <!-- v-bind:class (full syntax) -->
    <div v-bind:class="{ 'p-4': true }">
      v-bind:class
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

const isActive = ref(true);
const hasError = ref(false);
const loading = ref(false);

const activeClass = ref('is-active');
const errorClass = ref('text-red');

const computedClasses = computed(() => {
  return {
    'btn-primary': isActive.value,
    'rounded-lg': true
  };
});
</script>
