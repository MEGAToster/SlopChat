<script lang="ts">
  let { message }: { message: string } = $props();
</script>

<div class="toast" role="alert">{message}</div>

<style>
  .toast {
    position: absolute;
    right: 12px;
    bottom: 12px;
    left: 12px;
    padding: 8px 12px;
    border: 1px solid #7a3b3b;
    border-radius: 8px;
    background: #3a1d1d;
    color: #ffd9d9;
    font-size: 12px;
  }
</style>
