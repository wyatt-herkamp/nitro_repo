<template>
    <section :id="id + '-section'">
        <label :for="id">
            <slot />
        </label>
        <span class="fakeInput"> {{ value }}</span>
    </section>
</template>
<script setup lang="ts">
const props = defineProps({
    id: String,
    value: String
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
@import '@/assets/styles/form.scss';

.fakeInput {
    display: inline-block;
    width: 100%;
    padding: 0.375rem 0.75rem;
    font-size: 1rem;
    font-weight: 400;
    line-height: 1.5;
    border: 1px solid #ced4da;
    background-clip: padding-box;
    border-radius: 0.25rem;
    transition: border-color 0.15s ease-in-out, box-shadow 0.15s ease-in-out;
}
</style>