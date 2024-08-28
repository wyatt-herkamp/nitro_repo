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
.navLink {
  text-decoration: none;
  color: $text;
  font-weight: bold;
  padding: 0.5rem;
  // Align text vertically
  display: flex;
  align-items: center;
  gap: 0.5rem;
  // Box
  border-radius: 0.5rem;
  &:hover {
    background-color: $primary-70;
    transition: background-color 0.3s ease;
  }
}
.navLink[data-active='true'] {
  background-color: $primary-70;
  &:hover {
    cursor: default;
  }
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
