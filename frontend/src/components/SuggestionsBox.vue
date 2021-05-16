<template>
    <div id = "suggestions" :class = "{hidden: isSuggestHidden}"> </div>
</template>

<script>

export default {
    name: 'SuggestionsBox',
    props: {
        suggestions: Array,
        isSuggestHidden: Boolean
    }
};

</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>

#suggestions {
    height: 200px;
    width: 80%;
    max-width: 654px;
    min-width: 380px;

    background: #124;
    position: absolute;
    margin-left: auto;
    margin-right: auto;
    left: 0;
    right: 0;
    top: 435px;
    z-index: 0;
    border-radius: 0 0 20px 20px;
}
</style>
