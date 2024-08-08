<template>
  <button type="submit" v-bind="$attrs"><slot /></button>
</template>
<script setup lang="ts"></script>

<style scoped lang="scss">
@import '@/assets/styles/theme';

button {
  width: 100%;
  padding: 0.5rem;
  border-radius: 0.5rem;
  border: none;
  background-color: $secondary;
  color: #fff;
  font-weight: bold;
  cursor: pointer;
  &:hover {
    background-color: $secondary-90;
    transition: background-color;
    transition-duration: 0.5s;
    transition-timing-function: ease-in-out;
  }
}
button[disabled] {
  background-color: $secondary-70;
  color: #000;
  cursor: not-allowed;
}
.dangerButton {
  background-color: #ff0000;
  &:hover {
    background-color: #ff3333;
  }
}
</style>
