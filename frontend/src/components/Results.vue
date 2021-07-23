<template>
<div>
    <img alt="Q" id="Q" src="../assets/q.svg"/>
    <SearchBar/>
    <!-- If there is an error, it will be displayed in this div -->
    <div id = "error" :class = "{ hidden : err === null }"> {{ err }} </div>
    <ul id = "results-list">
        <li class = "result" v-for="(result, index) in results" :key = "index">
            <!-- &hellip; is only added if the text was truncated -->
            <a class = "result-title" :href="result.url"> {{ result.title === undefined || result.title === "" ? truncate(result.text, 25) + (result.text.length > 25 ? "&hellip;" : "") : result.title }} </a>
            <br/>
            <p> {{ result }} </p>
        </li>
    </ul>
</div>
</template>

<script>
import SearchBar from './SearchBar.vue';

export default  {
    name: 'Results',
    components: {
        SearchBar
    },
    data() {
        return {
            results: [],
            err: null
        }
    },
    beforeMount() {
        this.fetchResults();
    },
    methods: {
        fetchResults: function() {
            const query = this.$route.query.q;
            if (query === undefined || query === "") {
                console.error("Empty query.")
                // indicate error
                this.err = "Empty query";
                this.results = [];
            }
            else {
                this.err = null;
            }
            console.log("query:", query);

            // TODO also sort by rank
            fetch(`http://${window.location.hostname}:8000/query/${encodeURIComponent(query)}`, {
                    method: 'GET'
                })
                    .then(response => {
                        if (response.ok)
                            response.json()
                                .then(response => this.results = response)
                                .catch(err => console.error(err));
                        else
                            console.error("Suggest served responsed with an unexpected code: " + response.status)
                    }).catch(err => {
                        this.err = "Failed to fetch the query. Try again later.";
                        console.error(err);
            });
        },
        truncate(str, max_len) {
            return (str.length > max_len) ? str.substr(0, max_len - 1) : str;
        }
    },
    watch:{
        $route (){
            // whenever the url changes (meaning the query is different) fetch the new results from the server
            this.fetchResults();
        }
    }
    //beforeMount() {
    //    this.fetchResults();
    //}
}
</script>

<style scoped>
#error {
    /* color: #e4b9c3; */
    color: #a3c9ef;
    font-size: 2em;
    font-family: sans-serif, "Trebuchet MS";
}

#results-list {
    margin-top: 60px;
    display: inline-block;
}

.result {
    list-style: none;
}

.result-title {
    float: left;
    text-decoration: none;
    /* color: #5a55aa; */
    color: #5b5fe2;
}

.result-title:visited {
    color: #aeb1f1;
}

.result-title:hover {
    text-decoration: underline;
}

#Q {
    width: 50px;
    height: 50px;
    margin: 2.2em 0 0 1em;
    float: left;
}
</style>

<style>
/* SEARCH BAR */
/* TODO just a temporary solution.
    It would be better to have this css in SearchBar.vue and pass a variable
        to the search bar component to display the "results" page version when on the results page
        and the "start" page version when on the start page.
*/
#app {
    margin-top: 0;
}
.search_box {
    width: 70% !important;
    max-width: 500px !important;
    margin: 2.6em 2em 6em 1em;
    display: inline-block;
    position: absolute !important;
    float: left;
    left: 5.26em;
}
#suggestions {
    width: 70% !important;
    max-width: 571px !important;
    position: absolute !important;
    /* margin-left: 1.36em !important; - without the "Q" image */
    margin-left: 5.43em !important;
    top: 65px !important;
}
</style>
