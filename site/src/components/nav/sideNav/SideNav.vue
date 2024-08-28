<template>
  <nav>
    <slot />
  </nav>
</template>
<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
nav {
  margin: 0.5rem;
  margin-right: 0.5rem;
  border-top-right-radius: 8px;
  border-bottom-right-radius: 8px;
  width: 200px;
  background-color: $primary-50;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

#logoAndHome {
  img {
    width: 2rem;
    height: 2rem;
  }
  span {
    color: $text;
  }
}
.bottom {
  margin-top: auto;
}
</style>
