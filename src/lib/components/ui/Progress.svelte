<script lang="ts">
  export let value: number = 0;
  export let max: number = 100;
  export let class: string = "";

  $: percentage = Math.min(Math.max((value / max) * 100, 0), 100);
</script>

<div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 {class}">
  <div
    class="bg-blue-600 h-2.5 rounded-full transition-all duration-300 ease-in-out"
    style="width: {percentage}%"
  ></div>
</div>